//! OpenCode Session Output Reader Module
//!
//! Reads messages and output from OpenCode sessions via HTTP API.
//! Provides methods for retrieving session data in various formats.

use crate::Result;
use serde::{Deserialize, Serialize};

/// Message from OpenCode session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub tool_calls: Vec<ToolCall>,
}

/// Tool call within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub input: serde_json::Value,
}

/// OpenCode session output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutput {
    pub session_id: String,
    pub messages: Vec<OpenCodeMessage>,
    pub last_activity: String,
}

/// OpenCode output reader
pub struct OpenCodeReader {
    server_url: String,
}

impl OpenCodeReader {
    /// Create new output reader
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
        }
    }

    /// Get all messages from a session
    pub async fn get_messages(&self, session_id: &str, limit: Option<usize>) -> Result<Vec<OpenCodeMessage>> {
        use reqwest::Client;
        
        let url = format!("{}/session/{}/messages", self.server_url.trim_end_matches('/'), session_id);
        let mut client = Client::new();

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::TmuxError::Command(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::TmuxError::Command(format!(
                "OpenCode API error: {} - {}",
                status,
                error_text
            )));
        }

        let session_output: SessionOutput = response
            .json()
            .await
            .map_err(|e| crate::error::TmuxError::Command(format!("Failed to parse JSON: {}", e)))?;

        let mut messages = session_output.messages;
        
        // Apply limit if specified
        if let Some(n) = limit {
            messages.truncate(n);
        }

        Ok(messages)
    }

    /// Get last N messages from a session
    pub async fn tail_messages(&self, session_id: &str, n: usize) -> Result<Vec<OpenCodeMessage>> {
        self.get_messages(session_id, Some(n)).await
    }

    /// Watch session messages in real-time (polling)
    pub async fn watch_messages<F>(&self, session_id: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(&OpenCodeMessage) + Send,
    {
        use reqwest::Client;
        use std::time::Duration;
        
        let mut last_message_id = String::new();
        let mut client = Client::new();

        loop {
            let messages = self.get_messages(session_id, None).await?;
            
            for msg in &messages {
                // Only process new messages
                if msg.id != last_message_id {
                    callback(msg);
                    last_message_id = msg.id.clone();
                }
            }

            // Poll every second
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    }

    /// Get session output summary
    pub async fn get_session_summary(&self, session_id: &str) -> Result<SessionOutput> {
        use reqwest::Client;
        
        let url = format!("{}/session/{}", self.server_url.trim_end_matches('/'), session_id);
        let mut client = Client::new();

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::TmuxError::Command(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::TmuxError::Command(format!(
                "OpenCode API error: {} - {}",
                status,
                error_text
            )));
        }

        let session_output: SessionOutput = response
            .json()
            .await
            .map_err(|e| crate::error::TmuxError::Command(format!("Failed to parse JSON: {}", e)))?;

        Ok(session_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_creation() {
        let reader = OpenCodeReader::new("http://localhost:4096".to_string());
        assert_eq!(reader.server_url, "http://localhost:4096");
    }

    #[test]
    fn test_message_serialization() {
        let message = OpenCodeMessage {
            id: "msg_123".to_string(),
            session_id: "session_abc".to_string(),
            role: "user".to_string(),
            content: "Test message".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            tool_calls: vec![],
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("msg_123"));
        assert!(json.contains("Test message"));
    }
}
