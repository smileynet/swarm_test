use crate::Result;
use crate::opencode::{AgentResponse, MessageType, ToolCall};
use std::collections::HashMap;

pub struct OutputParser;

impl OutputParser {
    pub fn parse_agent_output(output: &str) -> Result<AgentResponse> {
        if Self::is_error(output) {
            return Ok(Self::parse_error(output));
        }

        if Self::is_tool_call(output) {
            return Ok(Self::parse_tool_call(output));
        }

        if Self::is_completion(output) {
            return Ok(Self::parse_completion(output));
        }

        Ok(Self::parse_message(output))
    }

    pub fn parse_multiple_outputs(output: &str) -> Vec<AgentResponse> {
        let mut responses = Vec::new();
        
        for segment in Self::split_into_segments(output) {
            if let Ok(response) = Self::parse_agent_output(&segment) {
                responses.push(response);
            }
        }

        responses
    }

    pub fn extract_tool_calls(response: &AgentResponse) -> Vec<ToolCall> {
        response.tool_calls.clone()
    }

    pub fn extract_errors(output: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        for line in output.lines() {
            if line.contains("<error>") || line.contains("Error:") || line.contains("error:") {
                errors.push(Self::strip_error_tags(line));
            }
        }

        errors
    }

    pub fn extract_json_blocks(output: &str) -> Vec<String> {
        let mut json_blocks = Vec::new();
        let mut in_json = false;
        let mut current_block = String::new();
        let mut brace_count = 0;

        for line in output.lines() {
            if line.trim().starts_with("{") || line.trim().starts_with("```json") {
                in_json = true;
            }

            if in_json {
                current_block.push_str(line);
                current_block.push('\n');

                brace_count += line.chars().filter(|&c| c == '{').count();
                brace_count -= line.chars().filter(|&c| c == '}').count();

                if brace_count == 0 && line.contains('}') {
                    json_blocks.push(current_block.trim().to_string());
                    current_block.clear();
                    in_json = false;
                }
            }

            if line.trim().contains("```") && in_json && brace_count == 0 {
                in_json = false;
            }
        }

        json_blocks
    }

    pub fn extract_code_blocks(output: &str) -> Vec<(String, String)> {
        let mut blocks = Vec::new();
        let mut in_block = false;
        let mut current_lang = String::new();
        let mut current_content = String::new();

        for line in output.lines() {
            if let Some(rest) = line.strip_prefix("```") {
                if in_block {
                    blocks.push((current_lang.clone(), current_content.trim().to_string()));
                    current_lang.clear();
                    current_content.clear();
                    in_block = false;
                } else {
                    in_block = true;
                    current_lang = rest.trim().to_string();
                }
            } else if in_block {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        blocks
    }

    pub fn extract_command_line(command: &str) -> Option<(String, Vec<String>)> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.is_empty() {
            return None;
        }

        let cmd = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        Some((cmd, args))
    }

    pub fn parse_timestamp(line: &str) -> Option<u64> {
        let patterns = [
            r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\]",
            r"\[(\d{10})\]",
            r"T(\d{10})",
        ];

        for pattern in &patterns {
            if let Some(captures) = regex_lite::Regex::new(pattern).ok()?.find(line) {
                let timestamp_str = captures.as_str();
                
                if timestamp_str.contains('-') {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", timestamp_str)) {
                        return Some(dt.timestamp() as u64);
                    }
                } else {
                    let digits: String = timestamp_str.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(ts) = digits.parse::<u64>() {
                        return Some(ts);
                    }
                }
            }
        }

        None
    }

    fn is_error(output: &str) -> bool {
        output.contains("<error>") || output.contains("Error:") || output.contains("error:")
    }

    fn is_tool_call(output: &str) -> bool {
        output.contains("Running:") || output.contains("<tool_call>")
    }

    fn is_completion(output: &str) -> bool {
        output.contains("<complete>") || output.contains("I'll complete") || output.contains("Done.")
    }

    fn parse_error(output: &str) -> AgentResponse {
        AgentResponse {
            success: false,
            message_type: MessageType::Error,
            content: Self::strip_error_tags(output),
            tool_calls: Vec::new(),
            error: Some(Self::extract_error_message(output)),
        }
    }

    fn parse_tool_call(output: &str) -> AgentResponse {
        let tool_calls = Self::extract_tool_calls_from_output(output);
        
        AgentResponse {
            success: true,
            message_type: MessageType::ToolCall,
            content: Self::extract_content_from_tool_output(output),
            tool_calls,
            error: None,
        }
    }

    fn parse_completion(output: &str) -> AgentResponse {
        AgentResponse {
            success: true,
            message_type: MessageType::Completion,
            content: Self::strip_completion_tags(output),
            tool_calls: Vec::new(),
            error: None,
        }
    }

    fn parse_message(output: &str) -> AgentResponse {
        AgentResponse {
            success: true,
            message_type: MessageType::Message,
            content: output.to_string(),
            tool_calls: Vec::new(),
            error: None,
        }
    }

    fn strip_error_tags(output: &str) -> String {
        output
            .replace("<error>", "")
            .replace("</error>", "")
            .lines()
            .filter(|line| !line.contains("Error:") && !line.contains("error:"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    fn extract_error_message(output: &str) -> String {
        output
            .lines()
            .filter(|line| line.contains("Error:") || line.contains("error:"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    fn extract_tool_calls_from_output(output: &str) -> Vec<ToolCall> {
        let mut calls = Vec::new();

        for line in output.lines() {
            if let Some(rest) = line.strip_prefix("Running: ") {
                if let Some((tool_name, args_str)) = Self::extract_command_line(rest) {
                    let mut args = HashMap::new();
                    args.insert("raw_args".to_string(), args_str.join(" "));
                    args.insert("command".to_string(), rest.to_string());
                    
                    calls.push(ToolCall {
                        tool_name,
                        arguments: args,
                    });
                }
            }

            if line.contains("<tool_call>") {
                if let Some(call) = Self::parse_xml_tool_call(line) {
                    calls.push(call);
                }
            }
        }

        calls
    }

    fn parse_xml_tool_call(line: &str) -> Option<ToolCall> {
        let name_start = line.find("<tool_call>")?;
        let name_end = line.find("</tool_call>")?;
        let content = &line[name_start + 11..name_end];
        
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let tool_name = parts[0].to_string();
        let mut args = HashMap::new();
        
        for part in &parts[1..] {
            if let Some((key, value)) = part.split_once('=') {
                args.insert(key.to_string(), value.trim_matches('"').to_string());
            }
        }

        Some(ToolCall { tool_name, arguments: args })
    }

    fn extract_content_from_tool_output(output: &str) -> String {
        output
            .lines()
            .filter(|line| !line.starts_with("Running:") && !line.starts_with("<tool_call>"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    fn strip_completion_tags(output: &str) -> String {
        output
            .replace("<complete>", "")
            .replace("</complete>", "")
            .lines()
            .filter(|line| !line.contains("I'll complete") && !line.contains("Done."))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    fn split_into_segments(output: &str) -> Vec<String> {
        let mut segments = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for line in output.lines() {
            if line.contains("<error>") || line.contains("<complete>") || line.contains("Running:") {
                if !current.is_empty() {
                    segments.push(current.clone());
                    current.clear();
                }
            }

            current.push_str(line);
            current.push('\n');
        }

        if !current.is_empty() {
            segments.push(current);
        }

        segments
    }
}

impl Default for OutputParser {
    fn default() -> Self {
        OutputParser
    }
}
