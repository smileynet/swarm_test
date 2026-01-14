//! Session ID Mapping Module
//!
//! Manages mappings between OpenCode session IDs and tmux session names.
//! Enables bridging between OpenCode protocol and tmux process control.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Mapping entry linking OpenCode session to tmux session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMapping {
    /// OpenCode session ID
    pub opencode_session_id: String,
    
    /// Corresponding tmux session name
    pub tmux_session_name: String,
    
    /// Timestamp when mapping was created
    pub created_at: String,
}

/// Session mapping store
pub struct SessionMappingStore {
    mappings: HashMap<String, SessionMapping>,
    storage_path: PathBuf,
}

impl SessionMappingStore {
    /// Create or load session mapping store
    pub fn new() -> Result<Self> {
        let storage_dir = dirs::home_dir()
            .ok_or_else(|| "Cannot find home directory".to_string())?
            .join(".swarm_test");
        
        fs::create_dir_all(&storage_dir)?;
        
        let storage_path = storage_dir.join("sessions.json");
        let mappings = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            mappings,
            storage_path,
        })
    }

    /// Insert a new session mapping
    pub fn insert(&mut self, opencode_id: String, tmux_name: String) -> Result<()> {
        let mapping = SessionMapping {
            opencode_session_id: opencode_id.clone(),
            tmux_session_name: tmux_name,
            created_at: format!("{:?}", SystemTime::now()),
        };
        
        self.mappings.insert(opencode_id, mapping);
        self.save()?;
        Ok(())
    }

    /// Lookup tmux session by OpenCode session ID
    pub fn lookup_tmux(&self, opencode_id: &str) -> Option<String> {
        self.mappings.get(opencode_id)
            .map(|m| m.tmux_session_name.clone())
    }

    /// Lookup OpenCode session ID by tmux session name
    pub fn lookup_opencode(&self, tmux_name: &str) -> Option<String> {
        self.mappings
            .iter()
            .find(|(_, m)| m.tmux_session_name == tmux_name)
            .map(|(k, _)| k.clone())
    }

    /// Remove a mapping
    pub fn remove(&mut self, opencode_id: &str) -> Result<()> {
        self.mappings.remove(opencode_id);
        self.save()?;
        Ok(())
    }

    /// List all mappings
    pub fn list(&self) -> Vec<SessionMapping> {
        self.mappings.values().cloned().collect()
    }

    /// Save mappings to disk
    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.mappings)
            .map_err(|e| format!("Failed to serialize mappings: {}", e))?;
        
        fs::write(&self.storage_path, content)
            .map_err(|e| format!("Failed to write mappings: {}", e))?;
        
        Ok(())
    }

    /// Clear all mappings
    pub fn clear(&mut self) -> Result<()> {
        self.mappings.clear();
        self.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = SessionMappingStore::new();
        assert_eq!(store.list().len(), 0);
    }

    #[test]
    fn test_insert_and_lookup() {
        let mut store = SessionMappingStore::new().unwrap();
        store.insert("session_abc".to_string(), "tmux_session_1".to_string()).unwrap();
        
        assert_eq!(store.lookup_tmux("session_abc"), Some("tmux_session_1".to_string()));
        assert_eq!(store.lookup_opencode("tmux_session_1"), Some("session_abc".to_string()));
    }

    #[test]
    fn test_remove() {
        let mut store = SessionMappingStore::new().unwrap();
        store.insert("session_xyz".to_string(), "tmux_session_2".to_string()).unwrap();
        store.remove("session_xyz").unwrap();
        
        assert_eq!(store.lookup_tmux("session_xyz"), None);
    }

    #[test]
    fn test_list() {
        let mut store = SessionMappingStore::new().unwrap();
        store.insert("session_1".to_string(), "tmux_1".to_string()).unwrap();
        store.insert("session_2".to_string(), "tmux_2".to_string()).unwrap();
        
        let mappings = store.list();
        assert_eq!(mappings.len(), 2);
        assert!(mappings.iter().any(|m| m.opencode_session_id == "session_1"));
    }

    #[test]
    fn test_clear() {
        let mut store = SessionMappingStore::new().unwrap();
        store.insert("session_test".to_string(), "tmux_test".to_string()).unwrap();
        store.clear().unwrap();
        
        assert_eq!(store.list().len(), 0);
    }
}
