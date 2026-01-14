use crate::opencode::{AgentResponse, MessageType};
use std::collections::HashMap;

pub struct MessageFilter;

impl MessageFilter {
    pub fn filter_by_type<'a>(
        messages: &'a [AgentResponse],
        message_type: MessageType,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| msg.message_type == message_type)
            .collect()
    }

    pub fn filter_by_success(messages: &[AgentResponse], success: bool) -> Vec<&AgentResponse> {
        messages
            .iter()
            .filter(|msg| msg.success == success)
            .collect()
    }

    pub fn filter_by_tool_name<'a>(
        messages: &'a [AgentResponse],
        tool_name: &str,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| {
                msg.tool_calls.iter().any(|call| call.tool_name == tool_name)
            })
            .collect()
    }

    pub fn filter_by_content<'a>(
        messages: &'a [AgentResponse],
        pattern: &str,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| msg.content.contains(pattern))
            .collect()
    }

    pub fn filter_by_time_range<'a>(
        messages: &'a [AgentResponse],
        start: u64,
        end: u64,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| {
                if let Some(timestamp) = Self::extract_timestamp(&msg.content) {
                    timestamp >= start && timestamp <= end
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn filter_by_error(messages: &[AgentResponse]) -> Vec<&AgentResponse> {
        messages
            .iter()
            .filter(|msg| !msg.success && msg.error.is_some())
            .collect()
    }

    pub fn filter_by_has_tool_calls(messages: &[AgentResponse]) -> Vec<&AgentResponse> {
        messages
            .iter()
            .filter(|msg| !msg.tool_calls.is_empty())
            .collect()
    }

    pub fn filter_by_file_operation<'a>(
        messages: &'a [AgentResponse],
        file_path: &str,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| {
                msg.content.contains(file_path) || 
                msg.tool_calls.iter().any(|call| {
                    call.arguments.values().any(|v| v.contains(file_path))
                })
            })
            .collect()
    }

    pub fn unique_tool_names(messages: &[AgentResponse]) -> Vec<String> {
        let mut names = HashMap::new();
        
        for msg in messages {
            for call in &msg.tool_calls {
                names.insert(call.tool_name.clone(), true);
            }
        }

        names.into_keys().collect()
    }

    pub fn count_by_type(messages: &[AgentResponse]) -> HashMap<MessageType, usize> {
        let mut counts = HashMap::new();
        
        for msg in messages {
            *counts.entry(msg.message_type.clone()).or_insert(0) += 1;
        }

        counts
    }

    pub fn count_by_tool(messages: &[AgentResponse]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for msg in messages {
            for call in &msg.tool_calls {
                *counts.entry(call.tool_name.clone()).or_insert(0) += 1;
            }
        }

        counts
    }

    pub fn find_first_error(messages: &[AgentResponse]) -> Option<&AgentResponse> {
        messages.iter().find(|msg| !msg.success && msg.error.is_some())
    }

    pub fn find_last_message(messages: &[AgentResponse]) -> Option<&AgentResponse> {
        messages.last()
    }

    pub fn extract_tool_arguments<'a>(
        messages: &'a [AgentResponse],
        tool_name: &str,
    ) -> Vec<&'a HashMap<String, String>> {
        messages
            .iter()
            .filter_map(|msg| {
                msg.tool_calls
                    .iter()
                    .find(|call| call.tool_name == tool_name)
                    .map(|call| &call.arguments)
            })
            .collect()
    }

    pub fn group_by_type(messages: &[AgentResponse]) -> HashMap<MessageType, Vec<&AgentResponse>> {
        let mut groups = HashMap::new();
        
        for msg in messages {
            groups
                .entry(msg.message_type.clone())
                .or_insert_with(Vec::new)
                .push(msg);
        }

        groups
    }

    pub fn extract_errors_only(messages: &[AgentResponse]) -> Vec<String> {
        messages
            .iter()
            .filter_map(|msg| msg.error.as_ref())
            .cloned()
            .collect()
    }

    pub fn extract_content_only(messages: &[AgentResponse]) -> Vec<String> {
        messages
            .iter()
            .map(|msg| msg.content.clone())
            .collect()
    }

    pub fn extract_timestamp(content: &str) -> Option<u64> {
        let patterns = [
            r"\[(\d{10})\]",
            r"\[(\d{13})\]",
            r"T(\d{10})",
        ];

        for pattern in &patterns {
            if let Some(re) = regex_lite::Regex::new(pattern).ok() {
                if let Some(captures) = re.find(content) {
                    let timestamp_str = captures.as_str();
                    let digits: String = timestamp_str
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect();
                    
                    if let Ok(ts) = digits.parse::<u64>() {
                        return Some(ts);
                    }
                }
            }
        }

        None
    }

    pub fn deduplicate_messages(messages: &[AgentResponse]) -> Vec<&AgentResponse> {
        let mut seen = HashMap::new();
        let mut unique = Vec::new();

        for msg in messages {
            let key = format!("{}:{}:{}", msg.message_type as u8, msg.success, msg.content.len());
            
            if !seen.contains_key(&key) {
                seen.insert(key, true);
                unique.push(msg);
            }
        }

        unique
    }

    pub fn sort_by_timestamp<'a>(
        messages: &'a [AgentResponse],
        ascending: bool,
    ) -> Vec<&'a AgentResponse> {
        let mut sorted: Vec<_> = messages.iter().collect();
        
        sorted.sort_by(|a, b| {
            let a_ts = Self::extract_timestamp(&a.content).unwrap_or(0);
            let b_ts = Self::extract_timestamp(&b.content).unwrap_or(0);
            
            if ascending {
                a_ts.cmp(&b_ts)
            } else {
                b_ts.cmp(&a_ts)
            }
        });

        sorted
    }

    pub fn limit_messages(messages: &[AgentResponse], n: usize) -> Vec<&AgentResponse> {
        messages.iter().take(n).collect()
    }

    pub fn paginate_messages(messages: &[AgentResponse], page: usize, page_size: usize) -> Vec<&AgentResponse> {
        let start = page.saturating_mul(page_size);
        let end = start.saturating_add(page_size);
        
        messages.iter().skip(start).take(end - start).collect()
    }

    pub fn search_by_pattern<'a>(
        messages: &'a [AgentResponse],
        regex: &str,
    ) -> Result<Vec<&'a AgentResponse>, regex_lite::Error> {
        let re = regex_lite::Regex::new(regex)?;
        
        Ok(messages
            .iter()
            .filter(|msg| re.is_match(&msg.content))
            .collect())
    }

    pub fn filter_multiline_content<'a>(
        messages: &'a [AgentResponse],
        min_lines: usize,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| msg.content.lines().count() >= min_lines)
            .collect()
    }

    pub fn filter_by_length<'a>(
        messages: &'a [AgentResponse],
        min_length: usize,
    ) -> Vec<&'a AgentResponse> {
        messages
            .iter()
            .filter(|msg| msg.content.len() >= min_length)
            .collect()
    }

    pub fn filter_empty_content(messages: &[AgentResponse]) -> Vec<&AgentResponse> {
        messages
            .iter()
            .filter(|msg| !msg.content.trim().is_empty())
            .collect()
    }
}
