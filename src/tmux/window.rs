use crate::tmux::cli::execute_command;
use crate::types::{Command, CommandTarget, ResponseData, SessionId, Window, WindowId};
use crate::{Result, TmuxError};

pub fn new_window(session_id: &SessionId, name: impl Into<String>) -> Result<Window> {
    let name = name.into();
    let cmd = Command {
        command: "new-window".to_string(),
        target: CommandTarget::Session(session_id.clone()),
        args: vec!["-n".to_string(), name.clone(), "-d".to_string()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to create window".to_string()),
        ));
    }

    let windows = list_windows(session_id)?;
    windows
        .into_iter()
        .find(|w| w.name == name)
        .ok_or_else(|| TmuxError::NotFound(format!("Window '{name}' not found after creation")))
}

pub fn kill_window(window_id: &WindowId) -> Result<()> {
    let cmd = Command {
        command: "kill-window".to_string(),
        target: CommandTarget::Window(window_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to kill window".to_string()),
        ));
    }

    Ok(())
}

pub fn list_windows(session_id: &SessionId) -> Result<Vec<Window>> {
    let cmd = Command {
        command: "list-windows".to_string(),
        target: CommandTarget::Session(session_id.clone()),
        args: vec![
            "-F".to_string(),
            "#{window_id}:#{window_name}:#{window_active}".to_string(),
        ],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to list windows".to_string()),
        ));
    }

    match response.data {
        ResponseData::Output(output) => {
            if output.trim().is_empty() {
                return Ok(Vec::new());
            }

            let mut windows = Vec::new();
            for line in output.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let id = WindowId(parts[0].to_string());
                    let name = parts[1].to_string();
                    let active: bool = parts[2] != "0";

                    let panes = super::pane::list_panes(&id)?;

                    windows.push(Window {
                        id,
                        session_id: session_id.clone(),
                        name,
                        panes,
                        active,
                    });
                }
            }

            Ok(windows)
        }
        _ => Ok(Vec::new()),
    }
}

pub fn get_window(window_id: &WindowId) -> Result<Window> {
    let session_id = extract_session_id(window_id)?;
    let windows = list_windows(&session_id)?;
    windows
        .into_iter()
        .find(|w| w.id == *window_id)
        .ok_or_else(|| TmuxError::NotFound(format!("Window '{}' not found", window_id.0)))
}

pub fn select_window(window_id: &WindowId) -> Result<()> {
    let cmd = Command {
        command: "select-window".to_string(),
        target: CommandTarget::Window(window_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to select window".to_string()),
        ));
    }

    Ok(())
}

pub fn rename_window(window_id: &WindowId, new_name: impl Into<String>) -> Result<()> {
    let new_name = new_name.into();
    let cmd = Command {
        command: "rename-window".to_string(),
        target: CommandTarget::Window(window_id.clone()),
        args: vec![new_name.clone()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to rename window".to_string()),
        ));
    }

    Ok(())
}

pub fn last_window(session_id: &SessionId) -> Result<()> {
    let cmd = Command {
        command: "last-window".to_string(),
        target: CommandTarget::Session(session_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to switch to last window".to_string()),
        ));
    }

    Ok(())
}

pub fn next_window(session_id: &SessionId) -> Result<()> {
    let cmd = Command {
        command: "next-window".to_string(),
        target: CommandTarget::Session(session_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to switch to next window".to_string()),
        ));
    }

    Ok(())
}

pub fn previous_window(session_id: &SessionId) -> Result<()> {
    let cmd = Command {
        command: "previous-window".to_string(),
        target: CommandTarget::Session(session_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(response.error.unwrap_or_else(|| {
            "Failed to switch to previous window".to_string()
        })));
    }

    Ok(())
}

fn extract_session_id(window_id: &WindowId) -> Result<SessionId> {
    window_id
        .0
        .split('@')
        .next()
        .map(|s| SessionId(s.to_string()))
        .ok_or_else(|| TmuxError::Command(format!("Invalid window ID format: {}", window_id.0)))
}
