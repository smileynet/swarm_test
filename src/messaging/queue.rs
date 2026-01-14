use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::types::{SessionId, PaneId, Message, Command};
use crate::error::TmuxError;
use crate::Result;
use super::send::{MessageSender, PromptMetadata};

const QUEUE_DIR: &str = ".opencode/queue";
const MESSAGE_FILE_SUFFIX: &str = ".msg";

#[derive(Debug, Clone)]
pub struct MessageQueue {
    base_path: PathBuf,
    queue: Arc<Mutex<VecDeque<QueuedMessage>>>,
    sender: MessageSender,
}

impl MessageQueue {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        let sender = MessageSender::new(&base_path);

        let queue_dir = base_path.join(QUEUE_DIR);
        fs::create_dir_all(&queue_dir)?;

        Ok(Self {
            base_path,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            sender,
        })
    }

    pub fn queue_dir(&self) -> PathBuf {
        self.base_path.join(QUEUE_DIR)
    }

    fn message_file(&self, session_id: &SessionId, message_id: &str) -> PathBuf {
        self.queue_dir().join(format!("{}-{}{}", session_id.0, message_id, MESSAGE_FILE_SUFFIX))
    }

    pub fn enqueue_message(&self, message: Message) -> Result<()> {
        let message_id = message.id.clone();
        let session_id = message.pane_id.0.clone();

        let queued = QueuedMessage {
            message,
            queued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            retries: 0,
        };

        let message_file = self.message_file(&SessionId(session_id.clone()), &message_id);

        if let Some(parent) = message_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&message_file)?;

        serde_json::to_writer_pretty(&mut file, &queued)?;
        file.flush()?;

        self.queue.lock().unwrap().push_back(queued);

        Ok(())
    }

    pub fn send_message(&self, session_id: &SessionId, content: &str) -> Result<String> {
        let message_id = uuid::Uuid::new_v4().to_string();
        let pane_id = PaneId(format!("{}:0.0", session_id.0));

        let message = Message {
            id: message_id.clone(),
            pane_id: pane_id.clone(),
            content: content.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.enqueue_message(message)?;

        Ok(message_id)
    }

    pub fn send_command(&self, session_id: &SessionId, command: &Command) -> Result<String> {
        let command_str = match command.target {
            crate::types::CommandTarget::Pane(ref pane_id) => {
                format!("send-keys -t {} {}", pane_id.0, command.command)
            }
            _ => return Err(TmuxError::InvalidState("Command target not supported".to_string())),
        };

        self.send_message(session_id, &command_str)
    }

    pub fn send_prompt_to_session(
        &self,
        session_id: &SessionId,
        prompt: &str,
        agent: &str,
    ) -> Result<String> {
        let pane_id = PaneId(format!("{}:0.0", session_id.0));
        let metadata = PromptMetadata::new(session_id.clone(), agent.to_string());

        self.sender.send_prompt_with_metadata(&pane_id, prompt, &metadata)?;

        self.send_message(session_id, prompt)
    }

    pub fn dequeue(&self) -> Option<QueuedMessage> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    pub fn peek(&self) -> Option<QueuedMessage> {
        let queue = self.queue.lock().unwrap();
        queue.front().cloned()
    }

    pub fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) -> Result<()> {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();

        let queue_dir = self.queue_dir();
        for entry in fs::read_dir(queue_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "msg").unwrap_or(false) {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    pub fn retry_failed(&self) -> Result<usize> {
        let queue_dir = self.queue_dir();
        let mut retry_count = 0;

        for entry in fs::read_dir(queue_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "msg").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                if let Ok(mut queued) = serde_json::from_str::<QueuedMessage>(&content) {
                    if queued.retries < 3 {
                        queued.retries += 1;
                        let mut file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&path)?;
                        serde_json::to_writer_pretty(&mut file, &queued)?;
                        file.flush()?;
                        self.queue.lock().unwrap().push_back(queued);
                        retry_count += 1;
                    }
                }
            }
        }

        Ok(retry_count)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueuedMessage {
    pub message: Message,
    pub queued_at: u64,
    pub retries: u32,
}

impl QueuedMessage {
    pub fn new(message: Message) -> Self {
        Self {
            message,
            queued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            retries: 0,
        }
    }

    pub fn id(&self) -> &str {
        &self.message.id
    }

    pub fn should_retry(&self) -> bool {
        self.retries < 3
    }
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending: usize,
    pub queued: usize,
    pub failed: usize,
}

impl MessageQueue {
    pub fn stats(&self) -> Result<QueueStats> {
        let pending = self.len();
        let mut queued = 0;
        let mut failed = 0;

        let queue_dir = self.queue_dir();
        for entry in fs::read_dir(queue_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "msg").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(queued_msg) = serde_json::from_str::<QueuedMessage>(&content) {
                        if queued_msg.retries >= 3 {
                            failed += 1;
                        } else {
                            queued += 1;
                        }
                    }
                }
            }
        }

        Ok(QueueStats {
            pending,
            queued,
            failed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::types::{PaneId, Message};

    #[test]
    fn test_enqueue_message() {
        let temp_dir = TempDir::new().unwrap();
        let queue = MessageQueue::new(temp_dir.path()).unwrap();

        let message = Message {
            id: "test_id".to_string(),
            pane_id: PaneId("test:0.0".to_string()),
            content: "Test message".to_string(),
            timestamp: 0,
        };

        queue.enqueue_message(message).unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_dequeue() {
        let temp_dir = TempDir::new().unwrap();
        let queue = MessageQueue::new(temp_dir.path()).unwrap();

        let message = Message {
            id: "test_id".to_string(),
            pane_id: PaneId("test:0.0".to_string()),
            content: "Test message".to_string(),
            timestamp: 0,
        };

        queue.enqueue_message(message).unwrap();
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.message.id, "test_id");
        assert!(queue.is_empty());
    }

    #[test]
    fn test_stats() {
        let temp_dir = TempDir::new().unwrap();
        let queue = MessageQueue::new(temp_dir.path()).unwrap();

        let message = Message {
            id: "test_id".to_string(),
            pane_id: PaneId("test:0.0".to_string()),
            content: "Test message".to_string(),
            timestamp: 0,
        };

        queue.enqueue_message(message).unwrap();
        let stats = queue.stats().unwrap();
        assert_eq!(stats.pending, 1);
    }
}
