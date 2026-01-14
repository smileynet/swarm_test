use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub enum TmuxError {
    Process(io::ErrorKind, String),
    Parse(String),
    Command(String),
    NotFound(String),
    InvalidState(String),
    Timeout,
    NotConnected,
}

impl fmt::Display for TmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TmuxError::Process(kind, msg) => write!(f, "Process error ({kind:?}): {msg}"),
            TmuxError::Parse(msg) => write!(f, "Parse error: {msg}"),
            TmuxError::Command(msg) => write!(f, "Command error: {msg}"),
            TmuxError::NotFound(target) => write!(f, "Not found: {target}"),
            TmuxError::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            TmuxError::Timeout => write!(f, "Operation timed out"),
            TmuxError::NotConnected => write!(f, "Not connected to tmux"),
        }
    }
}

impl std::error::Error for TmuxError {}

impl From<io::Error> for TmuxError {
    fn from(err: io::Error) -> Self {
        TmuxError::Process(err.kind(), err.to_string())
    }
}

impl From<serde_json::Error> for TmuxError {
    fn from(err: serde_json::Error) -> Self {
        TmuxError::Parse(err.to_string())
    }
}
