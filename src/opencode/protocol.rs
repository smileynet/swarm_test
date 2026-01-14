use crate::Result;
use crate::types::{Command, CommandTarget, PaneId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Protocol {
    fn format_command(&self, command: &Command) -> String;
    fn parse_response(&self, output: &str) -> Result<AgentResponse>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub message_type: MessageType,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    Message,
    ToolCall,
    Error,
    Completion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: HashMap<String, String>,
}

pub struct OpenCodeProtocol;

impl Protocol for OpenCodeProtocol {
    fn format_command(&self, command: &Command) -> String {
        match &command.target {
            CommandTarget::Server => format!("{} {}", command.command, command.args.join(" ")),
            CommandTarget::Session(session_id) => {
                format!("{} -t {} {}", command.command, session_id.0, command.args.join(" "))
            }
            CommandTarget::Window(window_id) => {
                format!("{} -t {} {}", command.command, window_id.0, command.args.join(" "))
            }
            CommandTarget::Pane(pane_id) => {
                format!("{} -t {} {}", command.command, pane_id.0, command.args.join(" "))
            }
        }
    }

    fn parse_response(&self, output: &str) -> Result<AgentResponse> {
        if output.contains("<error>") {
            return Ok(AgentResponse {
                success: false,
                message_type: MessageType::Error,
                content: extract_error_content(output),
                tool_calls: Vec::new(),
                error: Some(output.to_string()),
            });
        }

        let tool_calls = extract_tool_calls(output);

        if !tool_calls.is_empty() {
            Ok(AgentResponse {
                success: true,
                message_type: MessageType::ToolCall,
                content: extract_content(output),
                tool_calls,
                error: None,
            })
        } else if output.contains("<complete>") || output.contains("I'll complete this") {
            Ok(AgentResponse {
                success: true,
                message_type: MessageType::Completion,
                content: extract_content(output),
                tool_calls,
                error: None,
            })
        } else {
            Ok(AgentResponse {
                success: true,
                message_type: MessageType::Message,
                content: output.to_string(),
                tool_calls,
                error: None,
            })
        }
    }
}

fn extract_tool_calls(output: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    for line in output.lines() {
        if line.starts_with("Running: ") {
            if let Some(rest) = line.strip_prefix("Running: ") {
                if let Some(tool_name) = rest.split_whitespace().next() {
                    let args_str = rest.trim_start_matches(tool_name).trim();
                    let mut args = HashMap::new();
                    args.insert("raw_args".to_string(), args_str.to_string());
                    calls.push(ToolCall {
                        tool_name: tool_name.to_string(),
                        arguments: args,
                    });
                }
            }
        }
    }

    calls
}

fn extract_content(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("Running: ") && !line.starts_with("<") && !line.starts_with("</"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn extract_error_content(output: &str) -> String {
    output
        .replace("<error>", "")
        .replace("</error>", "")
        .trim()
        .to_string()
}

impl OpenCodeProtocol {
    pub fn new() -> Self {
        OpenCodeProtocol
    }

    pub fn format_agent_command(&self, prompt: &str, pane_id: &PaneId) -> Command {
        Command {
            command: "send-keys".to_string(),
            target: CommandTarget::Pane(pane_id.clone()),
            args: vec![
                prompt.to_string(),
                "Enter".to_string(),
            ],
        }
    }

    pub fn format_file_read_command(&self, path: &str) -> String {
        format!("Reading file: {}", path)
    }

    pub fn format_edit_command(&self, path: &str, old: &str, new: &str) -> String {
        format!(
            "Editing file: {}\nReplace: {}\nWith: {}",
            path,
            old.chars().take(50).collect::<String>(),
            new.chars().take(50).collect::<String>()
        )
    }

    pub fn format_write_command(&self, path: &str, content: &str) -> String {
        format!(
            "Writing file: {} ({} bytes)",
            path,
            content.len()
        )
    }
}

impl Default for OpenCodeProtocol {
    fn default() -> Self {
        Self::new()
    }
}
