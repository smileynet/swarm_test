//! OpenCode Server Discovery Module
//!
//! Discovers OpenCode server availability and configuration.
//! Provides health checking and automatic server detection.

use crate::Result;
use std::time::Duration;

/// OpenCode server status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStatus {
    /// Server is available and responding
    Available,
    /// Server is not running or unreachable
    Unavailable,
    /// Unknown status (check failed)
    Unknown,
}

/// OpenCode server detection result
#[derive(Debug, Clone)]
pub struct OpenCodeStatus {
    /// Whether server is running
    pub running: bool,
    /// Server URL (if found)
    pub url: Option<String>,
    /// Server health status
    pub status: ServerStatus,
}

/// OpenCode server discovery service
pub struct OpenCodeDiscovery {
    default_server_url: String,
}

impl Default for OpenCodeDiscovery {
    fn default() -> Self {
        Self {
            default_server_url: "http://127.0.0.1:4096".to_string(),
        }
    }
}

impl OpenCodeDiscovery {
    /// Create new discovery service
    pub fn new(default_server_url: String) -> Self {
        Self {
            default_server_url,
        }
    }

    /// Check if OpenCode server is running on default URL
    pub async fn check_default_server(&self) -> OpenCodeStatus {
        self.check_server(&self.default_server_url).await
    }

    /// Check if OpenCode server is running at specific URL
    pub async fn check_server(&self, server_url: &str) -> OpenCodeStatus {
        // Check if server URL is valid
        if server_url.is_empty() {
            return OpenCodeStatus {
                running: false,
                url: None,
                status: ServerStatus::Unknown,
            };
        }

        // Try to ping the server
        match reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
        {
            Ok(client) => {
                let health_url = format!("{}/health", server_url.trim_end_matches('/'));
                
                match client.get(&health_url).send().await {
                    Ok(response) => {
                        let status = if response.status().is_success() {
                            ServerStatus::Available
                        } else {
                            ServerStatus::Unavailable
                        };

                        OpenCodeStatus {
                            running: response.status().is_success(),
                            url: Some(server_url.to_string()),
                            status,
                        }
                    }
                    Err(_) => OpenCodeStatus {
                        running: false,
                        url: Some(server_url.to_string()),
                        status: ServerStatus::Unknown,
                    },
                }
            }
            Err(_) => OpenCodeStatus {
                running: false,
                url: Some(server_url.to_string()),
                status: ServerStatus::Unknown,
            },
        }
    }

    /// Discover OpenCode server automatically
    /// Tries common locations and environment variables
    pub async fn discover(&self) -> OpenCodeStatus {
        // Check default location first
        let default_status = self.check_default_server().await;
        
        if default_status.running {
            return default_status;
        }

        // Check environment variable
        if let Ok(env_url) = std::env::var("OPENCODE_SERVER_URL") {
            if !env_url.is_empty() {
                let env_status = self.check_server(&env_url).await;
                if env_status.running {
                    return env_status;
                }
            }
        }

        // Check common ports on localhost
        for port in [4096, 4097, 4098, 4099] {
            let url = format!("http://127.0.0.1:{}", port);
            let status = self.check_server(&url).await;
            if status.running {
                return status;
            }
        }

        // No server found
        OpenCodeStatus {
            running: false,
            url: None,
            status: ServerStatus::Unavailable,
        }
    }

    /// Get first available OpenCode server URL
    /// Returns configured URL if available, or discovered URL, or None
    pub async fn get_server_url(&self, configured_url: Option<String>) -> Option<String> {
        // Try configured URL first
        if let Some(url) = configured_url {
            if !url.is_empty() {
                let status = self.check_server(&url).await;
                if status.running {
                    return Some(url);
                }
            }
        }

        // Discover automatically
        let discovered = self.discover().await;
        
        if discovered.running {
            discovered.url
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_creation() {
        let discovery = OpenCodeDiscovery::new("http://localhost:4096".to_string());
        assert_eq!(discovery.default_server_url, "http://localhost:4096");
    }

    #[test]
    fn test_server_status_creation() {
        let status = OpenCodeStatus {
            running: true,
            url: Some("http://localhost:4096".to_string()),
            status: ServerStatus::Available,
        };
        
        assert!(status.running);
        assert_eq!(status.url, Some("http://localhost:4096".to_string()));
        assert_eq!(status.status, ServerStatus::Available);
    }

    #[test]
    fn test_status_variants() {
        assert_eq!(ServerStatus::Available, ServerStatus::Available);
        assert_ne!(ServerStatus::Available, ServerStatus::Unavailable);
        assert_ne!(ServerStatus::Unknown, ServerStatus::Available);
    }
}
