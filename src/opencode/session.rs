use crate::Result;
use crate::opencode::protocol::{AgentResponse, MessageType, OpenCodeProtocol, Protocol};
use crate::types::{PaneId, Session, SessionId};
use std::sync::{Arc, Mutex};

pub struct OpenCodeSession {
    pub session: Session,
    pub protocol: OpenCodeProtocol,
    pub agent_pane: Arc<Mutex<Option<PaneId>>>,
}

impl OpenCodeSession {
    pub fn new(session: Session) -> Self {
        OpenCodeSession {
            session,
            protocol: OpenCodeProtocol::new(),
            agent_pane: Arc::new(Mutex::new(None)),
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.session.id
    }

    pub fn name(&self) -> &str {
        &self.session.name
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn set_agent_pane(&self, pane_id: PaneId) {
        let mut guard = self.agent_pane.lock().unwrap();
        *guard = Some(pane_id);
    }

    pub fn get_agent_pane(&self) -> Option<PaneId> {
        let guard = self.agent_pane.lock().unwrap();
        guard.clone()
    }

    pub fn format_prompt(&self, prompt: &str) -> String {
        format!("{}\n", prompt)
    }

    pub fn parse_agent_output(&self, output: &str) -> Result<AgentResponse> {
        self.protocol.parse_response(output)
    }

    pub fn is_completion(&self, response: &AgentResponse) -> bool {
        response.message_type == MessageType::Completion
    }

    pub fn has_tool_calls(&self, response: &AgentResponse) -> bool {
        !response.tool_calls.is_empty()
    }

    pub fn is_error(&self, response: &AgentResponse) -> bool {
        response.message_type == MessageType::Error
    }
}

impl Clone for OpenCodeSession {
    fn clone(&self) -> Self {
        OpenCodeSession {
            session: self.session.clone(),
            protocol: OpenCodeProtocol::new(),
            agent_pane: self.agent_pane.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent_response() {
        use crate::opencode::protocol::Protocol;
        let protocol = OpenCodeProtocol::new();

        let tool_output = "I'll read the file\nRunning: read filePath=/path/to/file.txt";
        let response = protocol.parse_response(tool_output).unwrap();
        assert!(response.success);
        assert_eq!(response.message_type, MessageType::ToolCall);
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].tool_name, "read");
    }

    #[test]
    fn test_parse_error_response() {
        use crate::opencode::protocol::Protocol;
        let protocol = OpenCodeProtocol::new();

        let error_output = "<error>Failed to read file: permission denied</error>";
        let response = protocol.parse_response(error_output).unwrap();
        assert!(!response.success);
        assert_eq!(response.message_type, MessageType::Error);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_parse_completion_response() {
        use crate::opencode::protocol::Protocol;
        let protocol = OpenCodeProtocol::new();

        let completion_output = "I'll complete this task\n<complete>Task finished</complete>";
        let response = protocol.parse_response(completion_output).unwrap();
        assert!(response.success);
        assert_eq!(response.message_type, MessageType::Completion);
    }
}
