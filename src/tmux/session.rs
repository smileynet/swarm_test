use crate::tmux::cli::execute_command;
use crate::types::{Command, CommandTarget, ResponseData, Session, SessionId};
use crate::{Result, TmuxError};

pub fn new_session(name: impl Into<String>) -> Result<Session> {
    let name = name.into();
    let cmd = Command {
        command: "new-session".to_string(),
        target: CommandTarget::Server,
        args: vec!["-s".to_string(), name.clone(), "-d".to_string()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to create session".to_string()),
        ));
    }

    let sessions = list_sessions()?;
    sessions
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| TmuxError::NotFound(format!("Session '{name}' not found after creation")))
}

pub fn kill_session(id: &SessionId) -> Result<()> {
    let cmd = Command {
        command: "kill-session".to_string(),
        target: CommandTarget::Session(id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to kill session".to_string()),
        ));
    }

    Ok(())
}

pub fn list_sessions() -> Result<Vec<Session>> {
    let cmd = Command {
        command: "list-sessions".to_string(),
        target: CommandTarget::Server,
        args: vec![
            "-F".to_string(),
            "#{session_id}:#{session_name}:#{session_attached}".to_string(),
        ],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to list sessions".to_string()),
        ));
    }

    match response.data {
        ResponseData::Output(output) => {
            if output.trim().is_empty() {
                return Ok(Vec::new());
            }

            let mut sessions = Vec::new();
            for line in output.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let id = SessionId(parts[0].to_string());
                    let name = parts[1].to_string();
                    let attached: bool = parts[2] != "0";

                    let windows = super::window::list_windows(&id)?;

                    sessions.push(Session {
                        id,
                        name,
                        windows,
                        attached,
                    });
                }
            }

            Ok(sessions)
        }
        _ => Ok(Vec::new()),
    }
}

pub fn get_session(id: &SessionId) -> Result<Session> {
    let sessions = list_sessions()?;
    sessions
        .into_iter()
        .find(|s| s.id == *id)
        .ok_or_else(|| TmuxError::NotFound(format!("Session '{}' not found", id.0)))
}

pub fn attach_session(id: &SessionId) -> Result<()> {
    let cmd = Command {
        command: "attach-session".to_string(),
        target: CommandTarget::Session(id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to attach session".to_string()),
        ));
    }

    Ok(())
}

pub fn detach_session(id: &SessionId) -> Result<()> {
    let cmd = Command {
        command: "detach-client".to_string(),
        target: CommandTarget::Session(id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to detach session".to_string()),
        ));
    }

    Ok(())
}

pub fn rename_session(id: &SessionId, new_name: impl Into<String>) -> Result<()> {
    let new_name = new_name.into();
    let cmd = Command {
        command: "rename-session".to_string(),
        target: CommandTarget::Session(id.clone()),
        args: vec![new_name.clone()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to rename session".to_string()),
        ));
    }

    Ok(())
}
