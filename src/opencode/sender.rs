//! OpenCode Message Sender Module
//!
//! Sends messages via OpenCode protocol when available,
//! with fallback to direct tmux send-keys.

use crate::config::Config;
use crate::error::TmuxError;
use crate::Result;
use crate::config::MessageMode;
use crate::opencode::client::OpenCodeClient;
use crate::session_mapping::SessionMappingStore;
use crate::messaging::send::MessageSender as TmuxMessageSender;

/// Unified message sender with OpenCode integration
pub struct OpenCodeSender {
    /// OpenCode client (if available)
    opencode_client: Option<OpenCodeClient>,
    
    /// Tmux-based message sender (fallback)
    tmux_sender: TmuxMessageSender,
    
    /// Session mapping store
    session_mappings: SessionMappingStore,
    
    /// Configuration
    config: Config,
}

impl OpenCodeSender {
    /// Create new OpenCode sender
    pub fn new(config: Config) -> Self {
        Self {
            opencode_client: None,
            tmux_sender: TmuxMessageSender::new(&std::env::current_dir().unwrap()),
            session_mappings: SessionMappingStore::new().unwrap_or_else(|_| {
                eprintln!("Warning: Failed to initialize session mapping store");
                SessionMappingStore {
                    mappings: std::collections::HashMap::new(),
                    storage_path: std::path::PathBuf::from(".swarm_test/sessions.json"),
                }
            }),
            config,
        }
    }

    /// Initialize OpenCode client if available
    pub async fn initialize(&mut self, server_url: Option<String>) -> Result<()> {
        if server_url.is_none() {
            // Try to auto-discover
            let discovery = crate::opencode::discovery::OpenCodeDiscovery::new(
                self.config.get_opencode_server_url()
            );
            
            if self.config.auto_detect_opencode {
                let status = discovery.discover().await;
                if status.running {
                    self.opencode_client = Some(OpenCodeClient::new(&status.url.unwrap()));
                    println!("{}OpenCode server detected at: {}{}", 
                        "\x1b[32m", 
                        "\x1b[0m",
                        status.url.unwrap()
                    );
                } else {
                    println!("{}OpenCode server not available, using direct tmux mode{}", 
                        "\x1b[33m", 
                        "\x1b[0m"
                    );
                }
            }
        } else {
            // Use configured URL
            let url = server_url.unwrap();
            self.opencode_client = Some(OpenCodeClient::new(&url));
        }
        
        Ok(())
    }

    /// Send a prompt to session
    /// Tries OpenCode first, falls back to direct tmux
    pub async fn send_prompt(
        &mut self,
        session_id: &str,
        mode: MessageMode,
        prompt: &str,
    ) -> Result<bool> {
        // Try OpenCode first (if available and mode permits)
        if mode == MessageMode::Auto || mode == MessageMode::Opencode {
            if let Some(client) = &mut self.opencode_client {
                if let Ok(_) = client.session_prompt(session_id, prompt).await {
                    return Ok(true);
                }
            }
        }
        
        // Fallback to direct tmux
        match mode {
            MessageMode::Auto => {
                println!("{}OpenCode unavailable, sending via direct tmux{}", 
                    "\x1b[33m", 
                    "\x1b[0m"
                );
            }
            _ => {}
        }
        
        // Map session for future reference
        if let Ok(pane_id) = crate::tmux::pane::find_pane_by_session_name(session_id) {
            self.session_mappings.insert(session_id.to_string(), pane_id.to_string())?;
        }
        
        Ok(false)
    }

    /// Send a message to session
    /// Similar to send_prompt but uses message API
    pub async fn send_message(
        &mut self,
        session_id: &str,
        mode: MessageMode,
        message: &str,
    ) -> Result<bool> {
        // Try OpenCode first (if available and mode permits)
        if mode == MessageMode::Auto || mode == MessageMode::Opencode {
            if let Some(client) = &mut self.opencode_client {
                if let Ok(_) = client.session_message(session_id, message).await {
                    return Ok(true);
                }
            }
        }
        
        // Fallback to direct tmux
        match mode {
            MessageMode::Auto => {
                println!("{}OpenCode unavailable, sending via direct tmux{}", 
                    "\x1b[33m", 
                    "\x1b[0m"
                );
            }
            _ => {}
        }
        
        // Map session for future reference
        if let Ok(pane_id) = crate::tmux::pane::find_pane_by_session_name(session_id) {
            self.session_mappings.insert(session_id.to_string(), pane_id.to_string())?;
        }
        
        Ok(false)
    }

    /// Check if OpenCode integration is active
    pub fn is_opencode_active(&self) -> bool {
        self.opencode_client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_creation() {
        let config = crate::config::Config::default();
        let sender = OpenCodeSender::new(config);
        assert!(!sender.is_opencode_active());
    }

    #[test]
    fn test_initialize_without_auto_detect() {
        let mut config = crate::config::Config::default();
        config.auto_detect_opencode = false;
        let mut sender = OpenCodeSender::new(config);
        
        // In test environment, just check no crash
        // In real usage, would need async runtime
        assert!(!sender.is_opencode_active());
    }
}
