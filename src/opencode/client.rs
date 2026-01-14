//! OpenCode Client Module
//!
//! Minimal HTTP client for OpenCode Session API.
//! Uses OpenCode's HTTP protocol to send messages and retrieve session data.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenCode client for Session API
pub struct OpenCodeClient {
    server_url: String,
    client: reqwest::Client,
}

/// Session info from OpenCode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

/// OpenCode session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub session_id: String,
    pub status: String,
    pub message_count: usize,
}

/// Message from OpenCode session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

impl OpenCodeClient {
    /// Create new OpenCode client
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Send a prompt to OpenCode session
    pub async fn session_prompt(&self, session_id: &str, prompt: &str) -> Result<()> {
        let url = format!(
            "{}/session/{}/prompt",
            self.server_url.trim_end_matches('/'),
            session_id
        );

        #[derive(Serialize)]
        struct PromptRequest {
            content: String,
        }

        let body = PromptRequest {
            content: prompt.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::error::TmuxError::Command(format!("HTTP request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::error::TmuxError::Command(format!(
                "OpenCode API error: {} - {}",
                status, error_text
            )))
        }
    }

    /// Send a message to OpenCode session
    pub async fn session_message(&self, session_id: &str, message: &str) -> Result<()> {
        let url = format!(
            "{}/session/{}/message",
            self.server_url.trim_end_matches('/'),
            session_id
        );

        #[derive(Serialize)]
        struct MessageRequest {
            content: String,
        }

        let body = MessageRequest {
            content: message.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::error::TmuxError::Command(format!("HTTP request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::error::TmuxError::Command(format!(
                "OpenCode API error: {} - {}",
                status, error_text
            )))
        }
    }

    /// List all OpenCode sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let url = format!("{}/sessions", self.server_url.trim_end_matches('/'));

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                crate::error::TmuxError::Command(format!("HTTP request failed: {}", e))
            })?;

        if response.status().is_success() {
            let sessions: Vec<SessionInfo> = response.json().await.map_err(|e| {
                crate::error::TmuxError::Command(format!("Failed to parse JSON: {}", e))
            })?;
            Ok(sessions)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::error::TmuxError::Command(format!(
                "OpenCode API error: {} - {}",
                status, error_text
            )))
        }
    }

    /// Get session status
    pub async fn get_session_status(&self, session_id: &str) -> Result<SessionStatus> {
        let url = format!(
            "{}/session/{}/status",
            self.server_url.trim_end_matches('/'),
            session_id
        );

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                crate::error::TmuxError::Command(format!("HTTP request failed: {}", e))
            })?;

        if response.status().is_success() {
            let status: SessionStatus = response.json().await.map_err(|e| {
                crate::error::TmuxError::Command(format!("Failed to parse JSON: {}", e))
            })?;
            Ok(status)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::error::TmuxError::Command(format!(
                "OpenCode API error: {} - {}",
                status, error_text
            )))
        }
    }

    /// Check if OpenCode server is available
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.server_url.trim_end_matches('/'));

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                crate::error::TmuxError::Command(format!("HTTP request failed: {}", e))
            })?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenCodeClient::new("http://localhost:4096");
        assert_eq!(client.server_url, "http://localhost:4096");
    }

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "session_123".to_string(),
            name: "test_session".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("session_123"));
        assert!(json.contains("test_session"));
    }
}
