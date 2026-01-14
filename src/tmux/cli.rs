use crate::{Result, TmuxError};
use crate::types::{Command, CommandTarget, Response, ResponseData};
use std::process::{Command as StdCommand, Output};
use std::time::Duration;

pub struct TmuxCommand {
    args: Vec<String>,
    server: Option<String>,
    session: Option<String>,
    command: Option<String>,
}

impl TmuxCommand {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            server: None,
            session: None,
            command: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    pub fn server(mut self, server: impl Into<String>) -> Self {
        self.server = Some(server.into());
        self
    }

    pub fn session(mut self, session: impl Into<String>) -> Self {
        self.session = Some(session.into());
        self
    }

    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    fn build_args(&self) -> Vec<String> {
        let mut result = Vec::new();

        if let Some(ref server) = self.server {
            result.push("-L".to_string());
            result.push(server.clone());
        }

        if let Some(ref command) = self.command {
            result.push(command.clone());
        }

        if let Some(ref session) = self.session {
            result.push("-t".to_string());
            result.push(session.clone());
        }

        result.extend(self.args.clone());
        result
    }

    pub fn execute(&self) -> Result<Response> {
        let args = self.build_args();
        let output = StdCommand::new("tmux")
            .args(&args)
            .output()
            .map_err(|e| TmuxError::Process(e.kind(), format!("Failed to execute tmux command: {e}")))?;

        self.parse_output(output)
    }

    pub fn execute_with_timeout(&self, timeout: Duration) -> Result<Response> {
        let args = self.build_args();
        let mut child = StdCommand::new("tmux")
            .args(&args)
            .spawn()
            .map_err(|e| TmuxError::Process(e.kind(), format!("Failed to spawn tmux command: {e}")))?;

        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                child.kill().ok();
                return Err(TmuxError::Timeout);
            }

            match child.try_wait() {
                Ok(Some(_)) => {
                    let output = child
                        .wait_with_output()
                        .map_err(|e| TmuxError::Process(e.kind(), format!("Failed to get tmux output: {e}")))?;
                    return self.parse_output(output);
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    return Err(TmuxError::Process(e.kind(), format!("Failed to check process status: {e}")));
                }
            }
        }
    }

    fn parse_output(&self, output: Output) -> Result<Response> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let error_msg = if stderr.is_empty() {
                stdout.clone()
            } else {
                stderr.clone()
            };

            if error_msg.contains("not found") || error_msg.contains("no such") {
                Err(TmuxError::NotFound(error_msg))
            } else if error_msg.contains("not connected") || error_msg.contains("no server running") {
                Err(TmuxError::NotConnected)
            } else {
                Err(TmuxError::Command(error_msg))
            }
        } else {
            Ok(Response {
                success: true,
                data: if stdout.is_empty() {
                    ResponseData::Empty
                } else {
                    ResponseData::Output(stdout)
                },
                error: None,
            })
        }
    }
}

impl Default for TmuxCommand {
    fn default() -> Self {
        Self::new()
    }
}

pub fn execute_command(cmd: &Command) -> Result<Response> {
    let tmux_cmd = match &cmd.target {
        CommandTarget::Server => TmuxCommand::new()
            .command(cmd.command.clone())
            .args(&cmd.args),
        CommandTarget::Session(session_id) => {
            TmuxCommand::new()
                .command(cmd.command.clone())
                .session(session_id.0.clone())
                .args(&cmd.args)
        }
        CommandTarget::Window(window_id) => {
            TmuxCommand::new()
                .command(cmd.command.clone())
                .arg(format!("-t{}", window_id.0))
                .args(&cmd.args)
        }
        CommandTarget::Pane(pane_id) => {
            TmuxCommand::new()
                .command(cmd.command.clone())
                .arg(format!("-t{}", pane_id.0))
                .args(&cmd.args)
        }
    };

    tmux_cmd.execute()
}
