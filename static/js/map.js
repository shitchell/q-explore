// q-explore map.js - Leaflet map interactions

let map = null;
let centerMarker = null;
let radiusCircle = null;
let resultMarkers = [];

// Initialize map
function initMap() {
    // Create map centered on a default location
    map = L.map('map').setView([40.7128, -74.0060], 13);

    // Add tile layer (OpenStreetMap)
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
        maxZoom: 19,
    }).addTo(map);

    // Handle map click for location selection
    map.on('contextmenu', handleMapRightClick);
    map.on('click', handleMapClick);

    // Try to get user's location
    initUserLocation();

    // Set up "here" button
    elements.hereBtn.addEventListener('click', handleHereClick);
}

// Handle right-click to set location
function handleMapRightClick(e) {
    e.originalEvent.preventDefault();
    setMapCenter(e.latlng.lat, e.latlng.lng);
}

// Handle regular click (could show info or just select)
function handleMapClick(e) {
    // Could be used for selecting result markers in the future
}

// Initialize with user's location
async function initUserLocation() {
    const location = await getCurrentLocation();
    setMapCenter(location.lat, location.lng);
    map.setView([location.lat, location.lng], 13);
}

// Handle "here" button click
async function handleHereClick() {
    // First try browser geolocation
    if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
            (position) => {
                const lat = position.coords.latitude;
                const lng = position.coords.longitude;
                setMapCenter(lat, lng);
                map.setView([lat, lng], 13);
            },
            async (error) => {
                console.warn('Browser geolocation failed:', error);
                // Fall back to IP geolocation
                const location = await getCurrentLocation();
                setMapCenter(location.lat, location.lng);
                map.setView([location.lat, location.lng], 13);
            }
        );
    } else {
        // Fall back to IP geolocation
        const location = await getCurrentLocation();
        setMapCenter(location.lat, location.lng);
        map.setView([location.lat, location.lng], 13);
    }
}

// Set the center point for generation
function setMapCenter(lat, lng) {
    // Update state and display
    updateLocationDisplay(lat, lng);

    // Remove existing center marker
    if (centerMarker) {
        map.removeLayer(centerMarker);
    }

    // Create center marker
    const centerIcon = L.divIcon({
        className: 'marker-center',
        iconSize: [12, 12],
        iconAnchor: [6, 6],
    });

    centerMarker = L.marker([lat, lng], { icon: centerIcon }).addTo(map);

    // Update radius circle
    updateRadiusCircle();
}

// Update the radius circle display
function updateRadiusCircle() {
    if (!state.selectedLat || !state.selectedLng) return;

    // Always use meters for the Leaflet circle
    const radiusInMeters = getRadiusInMeters();

    // Remove existing circle
    if (radiusCircle) {
        map.removeLayer(radiusCircle);
    }

    // Create new circle
    radiusCircle = L.circle([state.selectedLat, state.selectedLng], {
        radius: radiusInMeters,
        color: '#00d9ff',
        fillColor: '#00d9ff',
        fillOpacity: 0.1,
        weight: 2,
    }).addTo(map);
}

// Clear all result markers
function clearResultMarkers() {
    resultMarkers.forEach(marker => map.removeLayer(marker));
    resultMarkers = [];
}

// Add a result marker to the map
function addResultMarker(lat, lng, type, zScore = null) {
    const className = `marker-${type.replace('_', '-')}`;

    const icon = L.divIcon({
        className: className,
        iconSize: [16, 16],
        iconAnchor: [8, 8],
    });

    const marker = L.marker([lat, lng], { icon: icon }).addTo(map);

    // Add popup with info
    let popupContent = `<strong>${formatTypeName(type)}</strong><br>`;
    popupContent += `${lat.toFixed(6)}, ${lng.toFixed(6)}`;
    if (zScore !== null) {
        popupContent += `<br>Z-score: ${zScore.toFixed(2)}`;
    }

    marker.bindPopup(popupContent);
    resultMarkers.push(marker);

    return marker;
}

// Display generation results on map
function displayResults(response, displayType) {
    clearResultMarkers();

    // Get the winner for the selected type
    const winner = response.winners[displayType];
    if (!winner) return;

    const result = winner.result;
    const lat = result.coords.lat;
    const lng = result.coords.lng;
    const zScore = result.z_score || null;

    // Add marker for the result
    const marker = addResultMarker(lat, lng, displayType, zScore);

    // Open popup and pan to result
    marker.openPopup();
    map.setView([lat, lng], map.getZoom());

    // If flower power mode, show all circle centers too
    if (response.circles.length > 1) {
        response.circles.forEach(circle => {
            if (circle.id !== 'center') {
                // Add smaller markers for petal centers
                const petalIcon = L.divIcon({
                    className: 'marker-center',
                    iconSize: [8, 8],
                    iconAnchor: [4, 4],
                });

                const petalMarker = L.marker(
                    [circle.center.lat, circle.center.lng],
                    { icon: petalIcon }
                ).addTo(map);

                resultMarkers.push(petalMarker);
            }
        });
    }
}

// Format type name for display
function formatTypeName(type) {
    const names = {
        'attractor': 'Attractor',
        'void': 'Void',
        'power': 'Power',
        'blind_spot': 'Blind Spot',
    };
    return names[type] || type;
}

// Listen for radius changes
document.addEventListener('DOMContentLoaded', () => {
    const radiusInput = document.getElementById('radius-input');
    if (radiusInput) {
        radiusInput.addEventListener('input', updateRadiusCircle);
        radiusInput.addEventListener('change', updateRadiusCircle);
    }
});
