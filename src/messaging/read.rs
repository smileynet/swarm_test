use crate::types::{SessionId, PaneId};
use crate::error::TmuxError;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const LOG_DIR: &str = "/tmp/tmux_logs";

pub struct LogReader {
    log_dir: PathBuf,
}

impl LogReader {
    pub fn new() -> Self {
        let log_dir = PathBuf::from(LOG_DIR);
        let _ = fs::create_dir_all(&log_dir);
        LogReader { log_dir }
    }

    pub fn with_dir(log_dir: PathBuf) -> Self {
        LogReader { log_dir }
    }

    pub fn read_log(&self, session_id: &SessionId) -> Result<String, TmuxError> {
        let log_path = self.session_log_path(session_id);
        
        if !log_path.exists() {
            return Ok(String::new());
        }

        Ok(fs::read_to_string(&log_path)
            .map_err(|e| TmuxError::Command(format!("Failed to read log file: {}", e)))?)
    }

    pub fn read_log_lines(&self, session_id: &SessionId) -> Result<Vec<String>, TmuxError> {
        let log_path = self.session_log_path(session_id);
        
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TmuxError::Command(format!("Failed to read log lines: {}", e)))?)
    }

    pub fn read_log_from(&self, session_id: &SessionId, offset: usize) -> Result<Vec<String>, TmuxError> {
        let lines = self.read_log_lines(session_id)?;
        Ok(lines.into_iter().skip(offset).collect())
    }

    pub fn read_pane_output(&self, pane_id: &PaneId) -> Result<String, TmuxError> {
        let pane_log_path = self.pane_log_path(pane_id);
        
        if !pane_log_path.exists() {
            return Ok(String::new());
        }

        Ok(fs::read_to_string(&pane_log_path)
            .map_err(|e| TmuxError::Command(format!("Failed to read pane log: {}", e)))?)
    }

    pub fn watch_log<F>(&self, session_id: &SessionId, callback: F) -> Result<(), TmuxError>
    where
        F: Fn(&str) + Send + Sync,
    {
        let log_path = self.session_log_path(session_id);
        
        if !log_path.exists() {
            return Err(TmuxError::NotFound(format!("Log file does not exist: {:?}", log_path)));
        }

        let file = File::open(&log_path)?;
        let mut reader = BufReader::new(file);

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Ok(_) => {
                    callback(&line);
                }
                Err(e) => {
                    return Err(TmuxError::Command(format!("Failed to read log: {}", e)));
                }
            }
        }
    }

    pub fn get_log_size(&self, session_id: &SessionId) -> Result<u64, TmuxError> {
        let log_path = self.session_log_path(session_id);
        
        if !log_path.exists() {
            return Ok(0);
        }

        Ok(log_path.metadata()
            .map(|m| m.len())
            .map_err(|e| TmuxError::Command(format!("Failed to get log size: {}", e)))?)
    }

    pub fn get_log_timestamp(&self, session_id: &SessionId) -> Result<u64, TmuxError> {
        let log_path = self.session_log_path(session_id);
        
        if !log_path.exists() {
            return Ok(0);
        }

        log_path.metadata()
            .map_err(|e| TmuxError::Command(format!("Failed to get log metadata: {}", e)))?
            .modified()
            .map_err(|e| TmuxError::Command(format!("Failed to get log timestamp: {}", e)))?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| TmuxError::Command(format!("Failed to convert timestamp: {}", e)))
            .map(|d| d.as_secs())
    }

    pub fn tail_log(&self, session_id: &SessionId, n: usize) -> Result<Vec<String>, TmuxError> {
        let lines = self.read_log_lines(session_id)?;
        let len = lines.len();
        
        if n >= len {
            return Ok(lines);
        }

        Ok(lines.into_iter().skip(len - n).collect())
    }

    pub fn search_log(&self, session_id: &SessionId, pattern: &str) -> Result<Vec<String>, TmuxError> {
        let lines = self.read_log_lines(session_id)?;
        
        Ok(lines.into_iter()
            .filter(|line| line.contains(pattern))
            .collect())
    }

    pub fn clear_log(&self, session_id: &SessionId) -> Result<(), TmuxError> {
        let log_path = self.session_log_path(session_id);
        
        fs::write(&log_path, "")
            .map_err(|e| TmuxError::Command(format!("Failed to clear log: {}", e)))
    }

    pub fn delete_log(&self, session_id: &SessionId) -> Result<(), TmuxError> {
        let log_path = self.session_log_path(session_id);
        
        if !log_path.exists() {
            return Ok(());
        }

        fs::remove_file(&log_path)
            .map_err(|e| TmuxError::Command(format!("Failed to delete log: {}", e)))
    }

    pub fn list_session_logs(&self) -> Result<Vec<SessionId>, TmuxError> {
        if !self.log_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.log_dir)
            .map_err(|e| TmuxError::Command(format!("Failed to read log directory: {}", e)))?;

        let mut sessions = Vec::new();
        
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("session_") && name.ends_with(".log") {
                    let session_id = name
                        .strip_prefix("session_")
                        .and_then(|s| s.strip_suffix(".log"))
                        .map(String::from);
                    
                    if let Some(id) = session_id {
                        sessions.push(SessionId(id));
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn session_log_path(&self, session_id: &SessionId) -> PathBuf {
        self.log_dir.join(format!("session_{}.log", session_id.0))
    }

    fn pane_log_path(&self, pane_id: &PaneId) -> PathBuf {
        self.log_dir.join(format!("pane_{}.log", pane_id.0))
    }
}

impl Default for LogReader {
    fn default() -> Self {
        Self::new()
    }
}
