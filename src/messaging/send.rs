use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::types::{PaneId, SessionId};
use crate::error::TmuxError;
use crate::Result;

const PROMPT_INPUT_SUFFIX: &str = ".prompt.input";
const PROMPT_DIR: &str = ".opencode/prompts";

#[derive(Debug, Clone)]
pub struct MessageSender {
    base_path: PathBuf,
}

impl MessageSender {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    fn prompt_dir(&self) -> PathBuf {
        self.base_path.join(PROMPT_DIR)
    }

    fn pane_prompt_file(&self, pane_id: &PaneId) -> PathBuf {
        self.prompt_dir().join(format!("{}{}", pane_id.0, PROMPT_INPUT_SUFFIX))
    }

    pub fn send_prompt(&self, pane_id: &PaneId, prompt: &str) -> Result<()> {
        let prompt_file = self.pane_prompt_file(pane_id);

        if let Some(parent) = prompt_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&prompt_file)?;

        writeln!(file, "{}", prompt)?;
        file.flush()?;

        Ok(())
    }

    pub fn send_prompt_with_metadata(
        &self,
        pane_id: &PaneId,
        prompt: &str,
        metadata: &PromptMetadata,
    ) -> Result<()> {
        let prompt_file = self.pane_prompt_file(pane_id);

        if let Some(parent) = prompt_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&prompt_file)?;

        writeln!(file, "# {}", metadata.session_id.0)?;
        writeln!(file, "# timestamp: {}", metadata.timestamp)?;
        writeln!(file, "# agent: {}", metadata.agent)?;
        writeln!(file, "")?;
        writeln!(file, "{}", prompt)?;
        file.flush()?;

        Ok(())
    }

    pub fn read_prompt(&self, pane_id: &PaneId) -> Result<String> {
        let prompt_file = self.pane_prompt_file(pane_id);

        if !prompt_file.exists() {
            return Err(TmuxError::NotFound(format!(
                "Prompt file not found for pane {}",
                pane_id.0
            )));
        }

        fs::read_to_string(&prompt_file).map_err(Into::into)
    }

    pub fn clear_prompt(&self, pane_id: &PaneId) -> Result<()> {
        let prompt_file = self.pane_prompt_file(pane_id);

        if prompt_file.exists() {
            fs::remove_file(&prompt_file)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PromptMetadata {
    pub session_id: SessionId,
    pub timestamp: u64,
    pub agent: String,
}

impl PromptMetadata {
    pub fn new(session_id: SessionId, agent: String) -> Self {
        Self {
            session_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            agent,
        }
    }
}

#[derive(Debug)]
pub struct FileLock {
    file: File,
}

impl FileLock {
    pub fn acquire<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;

        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let flock_result = unsafe {
                libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB)
            };
            if flock_result != 0 {
                return Err(TmuxError::Process(
                    std::io::ErrorKind::WouldBlock,
                    "File is locked".to_string(),
                ));
            }
        }

        #[cfg(not(unix))]
        {
            return Err(TmuxError::Process(
                std::io::ErrorKind::Unsupported,
                "File locking not supported on this platform".to_string(),
            ));
        }

        Ok(Self { file })
    }

    pub fn try_with<F, R, P: AsRef<Path>>(path: P, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        let _lock = Self::acquire(path)?;
        f()
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            unsafe {
                libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_send_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let sender = MessageSender::new(temp_dir.path());
        let pane_id = PaneId("test_pane".to_string());

        let prompt = "Hello, world!";
        sender.send_prompt(&pane_id, prompt).unwrap();

        let read_prompt = sender.read_prompt(&pane_id).unwrap();
        assert_eq!(read_prompt.trim(), prompt);
    }

    #[test]
    fn test_clear_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let sender = MessageSender::new(temp_dir.path());
        let pane_id = PaneId("test_pane".to_string());

        sender.send_prompt(&pane_id, "Test").unwrap();
        assert!(sender.read_prompt(&pane_id).is_ok());

        sender.clear_prompt(&pane_id).unwrap();
        assert!(sender.read_prompt(&pane_id).is_err());
    }
}
