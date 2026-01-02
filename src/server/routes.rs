//! HTTP API routes
//!
//! Defines all REST API endpoints for the server.

use crate::coord::flower::{generate, GenerationResponse};
use crate::coord::{available_types, AnomalyType, Coordinates, GenerationMode};
use crate::entropy::run_all_tests;
use crate::error::Error;
use crate::format::available_formats;
use crate::geo::{get_ip_locator, GeoLocation};
use crate::history::{History, HistoryEntry};
use crate::qrng::{available_backends, BackendInfo};
use crate::server::state::AppState;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::services::ServeDir;

/// Create the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    // Determine static files path
    // Try relative to cwd first, then fallback to common locations
    let static_path = if std::path::Path::new("static").exists() {
        "static".to_string()
    } else if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let path = exe_dir.join("static");
            if path.exists() {
                path.to_string_lossy().to_string()
            } else {
                "static".to_string()
            }
        } else {
            "static".to_string()
        }
    } else {
        "static".to_string()
    };

    Router::new()
        .route("/api/generate", post(generate_handler))
        .route("/api/status", get(status_handler))
        .route("/api/backends", get(backends_handler))
        .route("/api/types", get(types_handler))
        .route("/api/formats", get(formats_handler))
        .route("/api/location", get(location_handler))
        .route("/api/history", get(history_handler))
        .route("/api/history/:id", get(history_entry_handler).delete(history_delete_handler).patch(history_update_handler))
        .route("/api/share", post(create_share_handler))
        .nest_service("/", ServeDir::new(&static_path).append_index_html_on_directories(true))
        .with_state(state)
}

/// Generate request body
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lng: f64,
    /// Search radius in meters
    #[serde(default = "default_radius")]
    pub radius: f64,
    /// Number of points for analysis
    #[serde(default = "default_points")]
    pub points: usize,
    /// QRNG backend to use
    pub backend: Option<String>,
    /// Generation mode (standard or flower_power)
    #[serde(default)]
    pub mode: GenerationMode,
    /// Whether to include all generated points in response
    #[serde(default)]
    pub include_points: bool,
    /// Grid resolution for density analysis
    #[serde(default = "default_grid_resolution")]
    pub grid_resolution: usize,
}

fn default_radius() -> f64 {
    3000.0
}
fn default_points() -> usize {
    10_000
}
fn default_grid_resolution() -> usize {
    50
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let code = match &err {
            Error::InvalidCoordinates(_) => "INVALID_COORDINATES",
            Error::InvalidRadius(_) => "INVALID_RADIUS",
            Error::Qrng(_) => "QRNG_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            _ => "INTERNAL_ERROR",
        };
        ApiError {
            error: err.to_string(),
            code: code.to_string(),
        }
    }
}

/// Generate coordinates endpoint
///
/// POST /api/generate
async fn generate_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerationResponse>, ApiError> {
    // Validate coordinates
    let center = Coordinates::new(req.lat, req.lng);
    center.validate().map_err(ApiError::from)?;

    // Validate radius
    if req.radius <= 0.0 {
        return Err(ApiError {
            error: "Radius must be positive".to_string(),
            code: "INVALID_RADIUS".to_string(),
        });
    }

    // Get backend
    let backend_name = match &req.backend {
        Some(name) => name.clone(),
        None => state.backend_name().await,
    };
    let backend = crate::qrng::get_backend(&backend_name);

    // Generate
    let response = generate(
        center,
        req.radius,
        req.points,
        req.grid_resolution,
        req.include_points,
        req.mode,
        backend.name(),
        backend.as_ref(),
    )
    .map_err(ApiError::from)?;

    Ok(Json(response))
}

/// Status response
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Server is running
    pub running: bool,
    /// Server version
    pub version: String,
    /// Current backend
    pub backend: String,
    /// Entropy quality (if available)
    pub entropy_quality: Option<EntropyStatus>,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Entropy quality status
#[derive(Debug, Serialize, Deserialize)]
pub struct EntropyStatus {
    pub balanced: f64,
    pub uniform: f64,
    pub scattered: f64,
    pub overall: f64,
    pub passed: bool,
}

/// Server status endpoint
///
/// GET /api/status
async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    // Get current backend and test entropy quality
    let backend = state.get_backend().await;
    let backend_name = backend.name().to_string();

    // Generate some random bytes and test entropy
    let entropy_status = match backend.bytes(10_000) {
        Ok(bytes) => {
            let results = run_all_tests(&bytes);
            Some(EntropyStatus {
                balanced: results.balanced,
                uniform: results.uniform,
                scattered: results.scattered,
                overall: results.overall,
                passed: results.all_passed(),
            })
        }
        Err(_) => None,
    };

    Json(StatusResponse {
        running: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        backend: backend_name,
        entropy_quality: entropy_status,
        uptime_secs: 0, // TODO: track actual uptime
    })
}

/// Backends list response
#[derive(Debug, Serialize, Deserialize)]
pub struct BackendsResponse {
    pub backends: Vec<BackendInfo>,
    pub current: String,
}

/// List available QRNG backends
///
/// GET /api/backends
async fn backends_handler(State(state): State<Arc<AppState>>) -> Json<BackendsResponse> {
    let current = state.backend_name().await;
    Json(BackendsResponse {
        backends: available_backends(),
        current,
    })
}

/// Types list response
#[derive(Debug, Serialize, Deserialize)]
pub struct TypesResponse {
    pub types: Vec<TypeInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub description: String,
}

/// List available anomaly types
///
/// GET /api/types
async fn types_handler() -> Json<TypesResponse> {
    let types = available_types()
        .into_iter()
        .map(|t| TypeInfo {
            name: t.to_string(),
            description: match t {
                AnomalyType::BlindSpot => "Random point with no analysis".to_string(),
                AnomalyType::Attractor => "Densest cluster of points".to_string(),
                AnomalyType::Void => "Emptiest region".to_string(),
                AnomalyType::Power => "Most statistically anomalous".to_string(),
            },
        })
        .collect();

    Json(TypesResponse { types })
}

/// Formats list response
#[derive(Debug, Serialize, Deserialize)]
pub struct FormatsResponse {
    pub formats: Vec<FormatInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormatInfo {
    pub name: String,
    pub description: String,
}

/// List available output formats
///
/// GET /api/formats
async fn formats_handler() -> Json<FormatsResponse> {
    let formats = available_formats()
        .into_iter()
        .map(|f| FormatInfo {
            name: f.name,
            description: f.description,
        })
        .collect();

    Json(FormatsResponse { formats })
}

/// Get current location from IP address
///
/// GET /api/location
async fn location_handler() -> Result<Json<GeoLocation>, ApiError> {
    let locator = get_ip_locator();

    let location = locator.locate().await.map_err(|e| ApiError {
        error: e.to_string(),
        code: "LOCATION_ERROR".to_string(),
    })?;

    Ok(Json(location))
}

/// Share link request
#[derive(Debug, Deserialize)]
pub struct ShareRequest {
    pub lat: f64,
    pub lng: f64,
    pub radius: f64,
    pub mode: Option<String>,
    pub backend: Option<String>,
    #[serde(rename = "type")]
    pub anomaly_type: Option<String>,
}

/// Share link response
#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub url: String,
    pub params: String,
}

/// Create a share link
///
/// POST /api/share
async fn create_share_handler(
    Json(req): Json<ShareRequest>,
) -> Json<ShareResponse> {
    // Encode parameters as query string
    let mut params = format!("lat={}&lng={}&radius={}", req.lat, req.lng, req.radius);

    if let Some(mode) = &req.mode {
        params.push_str(&format!("&mode={}", urlencoding::encode(mode)));
    }

    if let Some(backend) = &req.backend {
        params.push_str(&format!("&backend={}", urlencoding::encode(backend)));
    }

    if let Some(t) = &req.anomaly_type {
        params.push_str(&format!("&type={}", urlencoding::encode(t)));
    }

    // Return just the params part - the frontend will construct the full URL
    Json(ShareResponse {
        url: format!("?{}", params),
        params,
    })
}

/// History list response
#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub entries: Vec<HistoryEntry>,
    pub count: usize,
}

/// Get history list
///
/// GET /api/history
async fn history_handler() -> Result<Json<HistoryResponse>, ApiError> {
    let history = History::load().map_err(|e| ApiError {
        error: e.to_string(),
        code: "HISTORY_ERROR".to_string(),
    })?;

    let count = history.len();
    let entries = history.entries().to_vec();

    Ok(Json(HistoryResponse { entries, count }))
}

/// Get a single history entry
///
/// GET /api/history/:id
async fn history_entry_handler(
    Path(id): Path<String>,
) -> Result<Json<HistoryEntry>, (StatusCode, Json<ApiError>)> {
    let history = History::load().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
            error: e.to_string(),
            code: "HISTORY_ERROR".to_string(),
        }))
    })?;

    history
        .get(&id)
        .cloned()
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(ApiError {
                error: format!("History entry not found: {}", id),
                code: "NOT_FOUND".to_string(),
            }))
        })
        .map(Json)
}

/// Delete a history entry
///
/// DELETE /api/history/:id
async fn history_delete_handler(
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let mut history = History::load().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
            error: e.to_string(),
            code: "HISTORY_ERROR".to_string(),
        }))
    })?;

    if history.remove(&id).is_some() {
        history.save().map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
                error: e.to_string(),
                code: "HISTORY_ERROR".to_string(),
            }))
        })?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiError {
            error: format!("History entry not found: {}", id),
            code: "NOT_FOUND".to_string(),
        })))
    }
}

/// Update history entry request
#[derive(Debug, Deserialize)]
pub struct UpdateHistoryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
}

/// Update a history entry
///
/// PATCH /api/history/:id
async fn history_update_handler(
    Path(id): Path<String>,
    Json(req): Json<UpdateHistoryRequest>,
) -> Result<Json<HistoryEntry>, (StatusCode, Json<ApiError>)> {
    let mut history = History::load().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
            error: e.to_string(),
            code: "HISTORY_ERROR".to_string(),
        }))
    })?;

    if !history.update_entry(&id, req.name, req.notes, req.favorite) {
        return Err((StatusCode::NOT_FOUND, Json(ApiError {
            error: format!("History entry not found: {}", id),
            code: "NOT_FOUND".to_string(),
        })));
    }

    history.save().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
            error: e.to_string(),
            code: "HISTORY_ERROR".to_string(),
        }))
    })?;

    // Get the updated entry to return
    let entry = history.get(&id).cloned().ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
            error: "Entry disappeared after update".to_string(),
            code: "INTERNAL_ERROR".to_string(),
        }))
    })?;

    Ok(Json(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn create_test_state() -> Arc<AppState> {
        Arc::new(AppState::new(crate::config::Config::default()))
    }

    #[tokio::test]
    async fn test_status_endpoint() {
        let state = create_test_state();
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/api/status").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let status: StatusResponse = serde_json::from_slice(&body).unwrap();

        assert!(status.running);
        assert_eq!(status.backend, "pseudo");
    }

    #[tokio::test]
    async fn test_backends_endpoint() {
        let state = create_test_state();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/backends")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let backends: BackendsResponse = serde_json::from_slice(&body).unwrap();

        assert!(!backends.backends.is_empty());
        assert_eq!(backends.current, "pseudo");
    }

    #[tokio::test]
    async fn test_types_endpoint() {
        let state = create_test_state();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/types")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let types: TypesResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(types.types.len(), 4);
    }

    #[tokio::test]
    async fn test_formats_endpoint() {
        let state = create_test_state();
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/formats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let formats: FormatsResponse = serde_json::from_slice(&body).unwrap();

        assert!(!formats.formats.is_empty());
    }

    #[tokio::test]
    async fn test_generate_endpoint() {
        let state = create_test_state();
        let app = create_router(state);

        let request_body = serde_json::json!({
            "lat": 40.7128,
            "lng": -74.0060,
            "radius": 1000.0,
            "points": 1000
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/generate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let gen: GenerationResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(gen.circles.len(), 1);
        assert!(gen.winners.contains_key(&AnomalyType::Attractor));
    }

    #[tokio::test]
    async fn test_generate_flower_power() {
        let state = create_test_state();
        let app = create_router(state);

        let request_body = serde_json::json!({
            "lat": 40.7128,
            "lng": -74.0060,
            "radius": 3000.0,
            "points": 1000,
            "mode": "flower_power"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/generate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let gen: GenerationResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(gen.circles.len(), 7);
    }

    #[tokio::test]
    async fn test_generate_invalid_coordinates() {
        let state = create_test_state();
        let app = create_router(state);

        let request_body = serde_json::json!({
            "lat": 91.0,  // Invalid latitude
            "lng": -74.0060,
            "radius": 1000.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/generate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let err: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(err.code, "INVALID_COORDINATES");
    }

    #[tokio::test]
    async fn test_generate_invalid_radius() {
        let state = create_test_state();
        let app = create_router(state);

        let request_body = serde_json::json!({
            "lat": 40.7128,
            "lng": -74.0060,
            "radius": -100.0  // Invalid radius
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/generate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let err: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(err.code, "INVALID_RADIUS");
    }
}
