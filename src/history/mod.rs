//! Generation history storage
//!
//! Stores and retrieves generation history from a file-based store.
//! History is stored in XDG data directory (~/.local/share/q-explore/).

use crate::coord::flower::GenerationResponse;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_DIR_NAME: &str = "q-explore";
const HISTORY_FILE_NAME: &str = "history.json";
const MAX_HISTORY_ENTRIES: usize = 100;

/// A history entry with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The generation response
    #[serde(flatten)]
    pub response: GenerationResponse,

    /// Optional name for this entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Whether this entry is marked as favorite
    #[serde(default)]
    pub favorite: bool,
}

impl HistoryEntry {
    /// Create a new history entry from a generation response
    pub fn new(response: GenerationResponse) -> Self {
        Self {
            response,
            name: None,
            notes: None,
            favorite: false,
        }
    }

    /// Set the name for this entry
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set notes for this entry
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Mark as favorite
    pub fn with_favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }
}

/// History storage manager
#[derive(Debug)]
pub struct History {
    entries: Vec<HistoryEntry>,
    path: PathBuf,
}

impl History {
    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join(APP_DIR_NAME))
            .ok_or_else(|| Error::Config("Could not determine data directory".to_string()))
    }

    /// Get the history file path
    pub fn history_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join(HISTORY_FILE_NAME))
    }

    /// Load history from disk
    pub fn load() -> Result<Self> {
        let path = Self::history_path()?;

        let entries = if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| {
                Error::Config(format!("Failed to read history file: {}", e))
            })?;

            serde_json::from_str(&content).map_err(|e| {
                Error::Config(format!("Failed to parse history file: {}", e))
            })?
        } else {
            Vec::new()
        };

        Ok(Self { entries, path })
    }

    /// Load history from a specific path (for testing)
    pub fn load_from(path: PathBuf) -> Result<Self> {
        let entries = if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| {
                Error::Config(format!("Failed to read history file: {}", e))
            })?;

            serde_json::from_str(&content).map_err(|e| {
                Error::Config(format!("Failed to parse history file: {}", e))
            })?
        } else {
            Vec::new()
        };

        Ok(Self { entries, path })
    }

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::Config(format!("Failed to create history directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(&self.entries).map_err(|e| {
            Error::Config(format!("Failed to serialize history: {}", e))
        })?;

        fs::write(&self.path, content).map_err(|e| {
            Error::Config(format!("Failed to write history file: {}", e))
        })?;

        Ok(())
    }

    /// Add a new entry to history
    ///
    /// Maintains max history size by removing oldest non-favorite entries
    pub fn add(&mut self, entry: HistoryEntry) {
        // Add to beginning (most recent first)
        self.entries.insert(0, entry);

        // Trim if over limit (preserve favorites)
        while self.entries.len() > MAX_HISTORY_ENTRIES {
            // Find oldest non-favorite entry to remove
            if let Some(idx) = self.entries.iter().rposition(|e| !e.favorite) {
                self.entries.remove(idx);
            } else {
                // All favorites, just remove the oldest
                self.entries.pop();
            }
        }
    }

    /// Add a generation response to history
    pub fn add_response(&mut self, response: GenerationResponse) {
        self.add(HistoryEntry::new(response));
    }

    /// Get all entries
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Get entry by ID
    pub fn get(&self, id: &str) -> Option<&HistoryEntry> {
        self.entries.iter().find(|e| e.response.id == id)
    }

    /// Get mutable entry by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut HistoryEntry> {
        self.entries.iter_mut().find(|e| e.response.id == id)
    }

    /// Remove entry by ID
    pub fn remove(&mut self, id: &str) -> Option<HistoryEntry> {
        if let Some(idx) = self.entries.iter().position(|e| e.response.id == id) {
            Some(self.entries.remove(idx))
        } else {
            None
        }
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get most recent entries
    pub fn recent(&self, count: usize) -> &[HistoryEntry] {
        &self.entries[..count.min(self.entries.len())]
    }

    /// Get favorite entries
    pub fn favorites(&self) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.favorite).collect()
    }

    /// Update entry metadata
    pub fn update_entry(
        &mut self,
        id: &str,
        name: Option<String>,
        notes: Option<String>,
        favorite: Option<bool>,
    ) -> bool {
        if let Some(entry) = self.get_mut(id) {
            if let Some(n) = name {
                entry.name = Some(n);
            }
            if let Some(n) = notes {
                entry.notes = Some(n);
            }
            if let Some(f) = favorite {
                entry.favorite = f;
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::flower::generate;
    use crate::coord::{Coordinates, GenerationMode};
    use crate::qrng::pseudo::SeededPseudoBackend;
    use tempfile::TempDir;

    fn create_test_response() -> GenerationResponse {
        let backend = SeededPseudoBackend::new(12345);
        let center = Coordinates::new(40.7128, -74.0060);
        generate(center, 1000.0, 100, 10, false, GenerationMode::Standard, "test", &backend)
            .unwrap()
    }

    fn create_test_history() -> (History, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_history.json");
        let history = History::load_from(path).unwrap();
        (history, temp_dir)
    }

    #[test]
    fn test_empty_history() {
        let (history, _temp) = create_test_history();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_add_entry() {
        let (mut history, _temp) = create_test_history();
        let response = create_test_response();
        let id = response.id.clone();

        history.add_response(response);

        assert_eq!(history.len(), 1);
        assert!(history.get(&id).is_some());
    }

    #[test]
    fn test_add_with_metadata() {
        let (mut history, _temp) = create_test_history();
        let response = create_test_response();
        let id = response.id.clone();

        let entry = HistoryEntry::new(response)
            .with_name("Test Location")
            .with_notes("Some notes")
            .with_favorite(true);
        history.add(entry);

        let retrieved = history.get(&id).unwrap();
        assert_eq!(retrieved.name, Some("Test Location".to_string()));
        assert_eq!(retrieved.notes, Some("Some notes".to_string()));
        assert!(retrieved.favorite);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_history.json");

        // Create and save history
        {
            let mut history = History::load_from(path.clone()).unwrap();
            let response = create_test_response();
            history.add_response(response);
            history.save().unwrap();
        }

        // Load and verify
        {
            let history = History::load_from(path).unwrap();
            assert_eq!(history.len(), 1);
        }
    }

    #[test]
    fn test_remove_entry() {
        let (mut history, _temp) = create_test_history();
        let response = create_test_response();
        let id = response.id.clone();

        history.add_response(response);
        assert_eq!(history.len(), 1);

        let removed = history.remove(&id);
        assert!(removed.is_some());
        assert!(history.is_empty());
    }

    #[test]
    fn test_recent_entries() {
        let (mut history, _temp) = create_test_history();

        // Add 5 entries
        for _ in 0..5 {
            history.add_response(create_test_response());
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);

        let recent = history.recent(10);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_favorites() {
        let (mut history, _temp) = create_test_history();

        // Add entries, some favorites
        for i in 0..5 {
            let entry = HistoryEntry::new(create_test_response()).with_favorite(i % 2 == 0);
            history.add(entry);
        }

        let favorites = history.favorites();
        assert_eq!(favorites.len(), 3); // 0, 2, 4 are favorites
    }

    #[test]
    fn test_update_entry() {
        let (mut history, _temp) = create_test_history();
        let response = create_test_response();
        let id = response.id.clone();

        history.add_response(response);

        let updated = history.update_entry(
            &id,
            Some("Updated Name".to_string()),
            None,
            Some(true),
        );
        assert!(updated);

        let entry = history.get(&id).unwrap();
        assert_eq!(entry.name, Some("Updated Name".to_string()));
        assert!(entry.favorite);
    }

    #[test]
    fn test_max_entries_limit() {
        let (mut history, _temp) = create_test_history();

        // Add more than max entries
        for _ in 0..150 {
            history.add_response(create_test_response());
        }

        assert!(history.len() <= MAX_HISTORY_ENTRIES);
    }

    #[test]
    fn test_favorites_preserved_on_trim() {
        let (mut history, _temp) = create_test_history();

        // Add a favorite entry
        let favorite = HistoryEntry::new(create_test_response()).with_favorite(true);
        let favorite_id = favorite.response.id.clone();
        history.add(favorite);

        // Add many non-favorite entries to trigger trim
        for _ in 0..MAX_HISTORY_ENTRIES + 10 {
            history.add_response(create_test_response());
        }

        // Favorite should still be there
        assert!(history.get(&favorite_id).is_some());
    }

    #[test]
    fn test_clear_history() {
        let (mut history, _temp) = create_test_history();

        for _ in 0..5 {
            history.add_response(create_test_response());
        }

        assert!(!history.is_empty());
        history.clear();
        assert!(history.is_empty());
    }

    #[test]
    fn test_history_entry_serialization() {
        let entry = HistoryEntry::new(create_test_response())
            .with_name("Test")
            .with_notes("Notes")
            .with_favorite(true);

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, Some("Test".to_string()));
        assert_eq!(parsed.notes, Some("Notes".to_string()));
        assert!(parsed.favorite);
    }
}
