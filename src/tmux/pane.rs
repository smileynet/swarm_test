use crate::tmux::cli::execute_command;
use crate::types::{Command, CommandTarget, Pane, PaneId, ResponseData, SessionId, WindowId};
use crate::{Result, TmuxError};

pub fn new_pane(window_id: &WindowId) -> Result<Pane> {
    let cmd = Command {
        command: "split-window".to_string(),
        target: CommandTarget::Window(window_id.clone()),
        args: vec!["-d".to_string(), "-v".to_string()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to create pane".to_string()),
        ));
    }

    let panes = list_panes(window_id)?;
    panes
        .into_iter()
        .last()
        .ok_or_else(|| TmuxError::NotFound("Pane not found after creation".to_string()))
}

pub fn split_pane_horizontal(window_id: &WindowId) -> Result<Pane> {
    let cmd = Command {
        command: "split-window".to_string(),
        target: CommandTarget::Window(window_id.clone()),
        args: vec!["-d".to_string(), "-h".to_string()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(response.error.unwrap_or_else(|| {
            "Failed to split pane horizontally".to_string()
        })));
    }

    let panes = list_panes(window_id)?;
    panes
        .into_iter()
        .last()
        .ok_or_else(|| TmuxError::NotFound("Pane not found after split".to_string()))
}

pub fn kill_pane(pane_id: &PaneId) -> Result<()> {
    let cmd = Command {
        command: "kill-pane".to_string(),
        target: CommandTarget::Pane(pane_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to kill pane".to_string()),
        ));
    }

        Ok(())
}

pub fn find_pane_by_session_name(session_name: &str) -> Result<Option<Pane>> {
    // Get all sessions and find the matching session
    use crate::tmux::session::list_sessions;
    
    let sessions = list_sessions()?;
    let matching_session = sessions.iter().find(|s| s.name == session_name);
    
    match matching_session {
        Some(session) => {
            // Get panes from the first window
            if let Some(first_window) = session.windows.first() {
                let panes = list_panes(first_window)?;
                return Ok(panes.first().cloned());
            }
        }
        None => Ok(None)
    }
}

pub fn send_keys(pane_id: &PaneId, keys: impl AsRef<str>) -> Result<()> {
    let keys = keys.as_ref();
    let cmd = Command {
        command: "send-keys".to_string(),
        target: CommandTarget::Pane(pane_id.clone()),
        args: vec![keys.to_string()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to send keys".to_string()),
        ));
    }

    Ok(())
}

pub fn send_keys_enter(pane_id: &PaneId, keys: impl AsRef<str>) -> Result<()> {
    send_keys(pane_id, keys)?;
    send_keys(pane_id, "Enter")?;
    Ok(())
}

pub fn capture_pane_output(pane_id: &PaneId) -> Result<String> {
    let cmd = Command {
        command: "capture-pane".to_string(),
        target: CommandTarget::Pane(pane_id.clone()),
        args: vec!["-p".to_string()],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to capture pane output".to_string()),
        ));
    }

    match response.data {
        ResponseData::Output(output) => Ok(output),
        _ => Ok(String::new()),
    }
}

pub fn capture_pane_start(pane_id: &PaneId, lines: usize) -> Result<String> {
    let cmd = Command {
        command: "capture-pane".to_string(),
        target: CommandTarget::Pane(pane_id.clone()),
        args: vec!["-p".to_string(), "-S".to_string(), format!("-{}", lines)],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to capture pane output".to_string()),
        ));
    }

    match response.data {
        ResponseData::Output(output) => Ok(output),
        _ => Ok(String::new()),
    }
}

pub fn select_pane(pane_id: &PaneId) -> Result<()> {
    let cmd = Command {
        command: "select-pane".to_string(),
        target: CommandTarget::Pane(pane_id.clone()),
        args: vec![],
    };

    let response = execute_command(&cmd)?;

    if !response.success {
        return Err(TmuxError::Command(
            response
                .error
                .unwrap_or_else(|| "Failed to select pane".to_string()),
        ));
    }

    Ok(())
}

pub fn resize_pane(pane_id: &PaneId, width: Option<usize>, height: Option<usize>) -> Result<()> {
    if let Some(w) = width {
        let cmd = Command {
            command: "resize-pane".to_string(),
            target: CommandTarget::Pane(pane_id.clone()),
            args: vec!["-x".to_string(), w.to_string()],
        };

        let response = execute_command(&cmd)?;

        if !response.success {
            return Err(TmuxError::Command(
                response
                    .error
                    .unwrap_or_else(|| "Failed to resize pane width".to_string()),
            ));
        }
    }

    if let Some(h) = height {
        let cmd = Command {
            command: "resize-pane".to_string(),
            target: CommandTarget::Pane(pane_id.clone()),
            args: vec!["-y".to_string(), h.to_string()],
        };

        let response = execute_command(&cmd)?;

        if !response.success {
            return Err(TmuxError::Command(
                response
                    .error
                    .unwrap_or_else(|| "Failed to resize pane height".to_string()),
            ));
        }
    }

    Ok(())
}

fn extract_session_id_from_window(window_id: &WindowId) -> Result<SessionId> {
    window_id
        .0
        .split('@')
        .next()
        .map(|s| SessionId(s.to_string()))
        .ok_or_else(|| TmuxError::Command(format!("Invalid window ID format: {}", window_id.0)))
}

fn extract_window_id(pane_id: &PaneId) -> Result<WindowId> {
    pane_id
        .0
        .split('%')
        .next()
        .map(|s| WindowId(s.to_string()))
        .ok_or_else(|| TmuxError::Command(format!("Invalid pane ID format: {}", pane_id.0)))
}
