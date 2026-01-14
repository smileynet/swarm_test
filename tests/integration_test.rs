use swarm_test::tmux::session as tmux_session;
use swarm_test::messaging::send::MessageSender;
use swarm_test::messaging::read::LogReader;
use swarm_test::types::{SessionId, PaneId};
use std::fs;
use std::path::PathBuf;

const TEST_SESSION_PREFIX: &str = "swarm_test_";
const TEST_LOG_DIR: &str = "/tmp/tmux_logs";

fn cleanup() {
    if let Ok(sessions) = tmux_session::list_sessions() {
        for session in sessions {
            if session.name.starts_with(TEST_SESSION_PREFIX) {
                let _ = tmux_session::kill_session(&session.id);
            }
        }
    }
    
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    let log_dir = PathBuf::from(TEST_LOG_DIR);
    if log_dir.exists() {
        if let Ok(entries) = fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("session_") {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

fn get_unique_session_name() -> String {
    format!("{}{}", TEST_SESSION_PREFIX, uuid::Uuid::new_v4())
}

#[test]
fn test_full_session_lifecycle() {
    cleanup();
    
    let session_name = get_unique_session_name();
    
    let session = tmux_session::new_session(&session_name)
        .expect("Failed to create session");
    
    let session_id = session.id.clone();
    
    assert!(!session.id.0.is_empty());
    assert_eq!(session.name, session_name);
    assert!(!session.windows.is_empty());
    
    let retrieved_session = tmux_session::get_session(&session.id)
        .expect("Failed to retrieve session");
    
    assert_eq!(session.id, retrieved_session.id);
    assert_eq!(session.name, retrieved_session.name);
    
    let all_sessions = tmux_session::list_sessions()
        .expect("Failed to list sessions");
    
    let found = all_sessions.iter()
        .any(|s| s.id == session.id);
    assert!(found, "Session not found in list");
    
    let new_name = format!("{}_renamed", session_name);
    let rename_result = tmux_session::rename_session(&session.id, &new_name);
    
    if rename_result.is_ok() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        if let Ok(all_sessions) = tmux_session::list_sessions() {
            if let Some(renamed_session) = all_sessions.iter().find(|s| s.id == session_id) {
                assert_eq!(renamed_session.name, new_name, "Session name should be updated to {}", new_name);
            }
        }
    } else {
        eprintln!("Warning: Session rename not supported in this environment: {:?}", rename_result);
    }
    
    let kill_result = tmux_session::kill_session(&session.id);
    if kill_result.is_err() {
        eprintln!("Warning: Failed to kill session {}, trying cleanup: {:?}", session.id.0, kill_result);
    }
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let result = tmux_session::get_session(&session.id);
    assert!(result.is_err(), "Session should not exist after kill");
    
    cleanup();
}

#[test]
fn test_message_sending_workflow() {
    cleanup();
    
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();
    
    let sender = MessageSender::new(base_path);
    let pane_id = PaneId("test_pane_1".to_string());
    
    let test_message = "Hello, OpenCode!";
    
    sender.send_prompt(&pane_id, test_message)
        .expect("Failed to send prompt");
    
    let prompt_dir = base_path.join(".opencode/prompts");
    let prompt_file = prompt_dir.join(format!("{}.prompt.input", pane_id.0));
    
    assert!(prompt_file.exists(), "Prompt file should exist");
    
    let read_message = sender.read_prompt(&pane_id)
        .expect("Failed to read prompt");
    
    assert_eq!(read_message.trim(), test_message);
    
    sender.clear_prompt(&pane_id)
        .expect("Failed to clear prompt");
    
    assert!(!prompt_file.exists(), "Prompt file should be deleted");
}

#[test]
fn test_message_with_metadata() {
    cleanup();
    
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();
    
    let sender = MessageSender::new(base_path);
    let pane_id = PaneId("test_pane_2".to_string());
    let session_id = SessionId("$0".to_string());
    
    use swarm_test::messaging::send::PromptMetadata;
    let metadata = PromptMetadata::new(session_id.clone(), "TestAgent".to_string());
    
    let test_message = "Test prompt with metadata";
    
    sender.send_prompt_with_metadata(&pane_id, test_message, &metadata)
        .expect("Failed to send prompt with metadata");
    
    let read_message = sender.read_prompt(&pane_id)
        .expect("Failed to read prompt");
    
    assert!(read_message.contains(&format!("# {}", session_id.0)));
    assert!(read_message.contains("# timestamp:"));
    assert!(read_message.contains("# agent: TestAgent"));
    assert!(read_message.contains(test_message));
}

#[test]
fn test_log_reading_workflow() {
    cleanup();
    
    let log_dir = PathBuf::from(TEST_LOG_DIR);
    fs::create_dir_all(&log_dir).expect("Failed to create log dir");
    
    let session_id = SessionId("$0".to_string());
    let log_path = log_dir.join(format!("session_{}.log", session_id.0));
    
    let test_lines = vec![
        "Line 1: Session started",
        "Line 2: Processing request",
        "Line 3: Task completed",
    ];
    
    fs::write(&log_path, test_lines.join("\n") + "\n")
        .expect("Failed to write log file");
    
    let reader = LogReader::new();
    
    let log_content = reader.read_log(&session_id)
        .expect("Failed to read log");
    
    for line in &test_lines {
        assert!(log_content.contains(line));
    }
    
    let log_lines = reader.read_log_lines(&session_id)
        .expect("Failed to read log lines");
    
    assert_eq!(log_lines.len(), test_lines.len());
    
    let last_n = reader.tail_log(&session_id, 2)
        .expect("Failed to tail log");
    
    assert_eq!(last_n.len(), 2);
    assert_eq!(last_n[0], test_lines[1]);
    assert_eq!(last_n[1], test_lines[2]);
    
    let search_results = reader.search_log(&session_id, "Processing")
        .expect("Failed to search log");
    
    assert_eq!(search_results.len(), 1);
    assert!(search_results[0].contains("Processing"));
    
    let log_size = reader.get_log_size(&session_id)
        .expect("Failed to get log size");
    
    assert!(log_size > 0);
    
    reader.clear_log(&session_id)
        .expect("Failed to clear log");
    
    let cleared_content = reader.read_log(&session_id)
        .expect("Failed to read cleared log");
    
    assert!(cleared_content.is_empty());
    
    fs::remove_file(&log_path).ok();
}

#[test]
fn test_session_list_empty() {
    cleanup();
    
    let sessions = tmux_session::list_sessions()
        .expect("Failed to list sessions");
    
    let test_sessions: Vec<_> = sessions.iter()
        .filter(|s| s.name.starts_with(TEST_SESSION_PREFIX))
        .collect();
    
    assert!(test_sessions.is_empty(), "No test sessions should exist after cleanup. Found: {:?}", test_sessions);
}

#[test]
fn test_multiple_sessions() {
    cleanup();
    
    let session1_name = get_unique_session_name();
    let session2_name = get_unique_session_name();
    
    let session1 = tmux_session::new_session(&session1_name)
        .expect("Failed to create first session");
    let session2 = tmux_session::new_session(&session2_name)
        .expect("Failed to create second session");
    
    assert_ne!(session1.id, session2.id);
    
    let all_sessions = tmux_session::list_sessions()
        .expect("Failed to list sessions");
    
    let found_session1 = all_sessions.iter().find(|s| s.id == session1.id);
    let found_session2 = all_sessions.iter().find(|s| s.id == session2.id);
    
    assert!(found_session1.is_some(), "First session not found");
    assert!(found_session2.is_some(), "Second session not found");
    
    tmux_session::kill_session(&session1.id).ok();
    tmux_session::kill_session(&session2.id).ok();
    
    cleanup();
}

#[test]
fn test_log_reader_with_custom_dir() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let custom_log_dir = temp_dir.path().join("logs");
    
    let reader = LogReader::with_dir(custom_log_dir.clone());
    
    let session_id = SessionId("custom_test".to_string());
    let log_path = custom_log_dir.join(format!("session_{}.log", session_id.0));
    
    fs::create_dir_all(&custom_log_dir).expect("Failed to create log dir");
    fs::write(&log_path, "Custom log content").expect("Failed to write log");
    
    let content = reader.read_log(&session_id)
        .expect("Failed to read log from custom dir");
    
    assert_eq!(content.trim(), "Custom log content");
}

#[test]
fn test_nonexistent_session_error() {
    let nonexistent_id = SessionId("$999999".to_string());
    
    let result = tmux_session::get_session(&nonexistent_id);
    assert!(result.is_err());
}

#[test]
fn test_nonexistent_log_handling() {
    cleanup();
    
    let reader = LogReader::new();
    let nonexistent_id = SessionId("$nonexistent".to_string());
    
    let content = reader.read_log(&nonexistent_id)
        .expect("Should return empty string for nonexistent log");
    
    assert!(content.is_empty());
    
    let lines = reader.read_log_lines(&nonexistent_id)
        .expect("Should return empty vec for nonexistent log");
    
    assert!(lines.is_empty());
}

#[test]
fn test_message_sender_base_path() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();
    
    let sender = MessageSender::new(base_path);
    
    assert_eq!(sender.base_path(), base_path);
}

#[test]
fn test_prompt_directory_creation() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();
    
    let sender = MessageSender::new(base_path);
    let pane_id = PaneId("test_pane_dir".to_string());
    
    sender.send_prompt(&pane_id, "Test")
        .expect("Failed to send prompt");
    
    let prompt_dir = base_path.join(".opencode/prompts");
    assert!(prompt_dir.exists());
}
