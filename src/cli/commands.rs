use super::main::colors;
use std::io::Write;
use std::path::PathBuf;
use swarm_test::Result;
use swarm_test::messaging::read::LogReader;
use swarm_test::messaging::send::MessageSender;
use swarm_test::tmux::session as tmux_session;
use swarm_test::types::{PaneId, SessionId};

pub fn print_usage() {
    println!("{}Swarm Control CLI{}", colors::bold(), colors::reset());
    println!();
    println!("{}Usage:{}", colors::cyan(), colors::reset());
    println!(
        "  {}swarm_test{} <command> [args]",
        colors::yellow(),
        colors::reset()
    );
    println!();
    println!("{}Commands:{}", colors::cyan(), colors::reset());
    println!(
        "  {}session{} <subcommand>     Session management",
        colors::green(),
        colors::reset()
    );
    println!(
        "    {}start{} <name>            Create a new session",
        colors::yellow(),
        colors::reset()
    );
    println!(
        "    {}stop{} <id|name>          Stop a session",
        colors::yellow(),
        colors::reset()
    );
    println!(
        "    {}list{}                    List all sessions",
        colors::yellow(),
        colors::reset()
    );
    println!(
        "    {}attach{} <id|name>        Attach to a session",
        colors::yellow(),
        colors::reset()
    );
    println!(
        "    {}detach{} <id|name>        Detach from a session",
        colors::yellow(),
        colors::reset()
    );
    println!();
    println!(
        "  {}message{} <subcommand>     Message management",
        colors::green(),
        colors::reset()
    );
    println!(
        "    {}send{} <pane_id> <msg>    Send message to pane",
        colors::yellow(),
        colors::reset()
    );
    println!();
    println!(
        "  {}output{} <subcommand>      Output management",
        colors::green(),
        colors::reset()
    );
    println!(
        "    {}read{} <session_id>       Read session output",
        colors::yellow(),
        colors::reset()
    );
    println!(
        "    {}tail{} <session_id> [n]   Tail last N lines",
        colors::yellow(),
        colors::reset()
    );
    println!(
        "    {}watch{} <session_id>      Watch session output live",
        colors::yellow(),
        colors::reset()
    );
    println!();
    println!(
        "  {}status{}                   Show system status",
        colors::green(),
        colors::reset()
    );
    println!();
    println!(
        "  {}help{}                     Show this help",
        colors::green(),
        colors::reset()
    );
}

pub fn handle_session(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session subcommand required{}",
            colors::red(),
            colors::reset()
        );
        eprintln!("  session start <name>");
        eprintln!("  session stop <id|name>");
        eprintln!("  session list");
        eprintln!("  session attach <id|name>");
        eprintln!("  session detach <id|name>");
        std::process::exit(1);
    }

    let subcommand = &args[0];
    let subcommand_args = &args[1..];

    match subcommand.as_str() {
        "start" => session_start(subcommand_args),
        "stop" => session_stop(subcommand_args),
        "list" => session_list(subcommand_args),
        "attach" => session_attach(subcommand_args),
        "detach" => session_detach(subcommand_args),
        _ => {
            eprintln!(
                "{}Unknown session subcommand: '{}'",
                colors::red(),
                subcommand
            );
            std::process::exit(1);
        }
    }
}

pub fn handle_message(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: message subcommand required{}",
            colors::red(),
            colors::reset()
        );
        eprintln!("  message send <pane_id> <message>");
        std::process::exit(1);
    }

    let subcommand = &args[0];
    let subcommand_args = &args[1..];

    match subcommand.as_str() {
        "send" => message_send(subcommand_args),
        _ => {
            eprintln!(
                "{}Unknown message subcommand: '{}'",
                colors::red(),
                subcommand
            );
            std::process::exit(1);
        }
    }
}

pub fn handle_output(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: output subcommand required{}",
            colors::red(),
            colors::reset()
        );
        eprintln!("  output read <session_id>");
        eprintln!("  output tail <session_id> [n]");
        eprintln!("  output watch <session_id>");
        std::process::exit(1);
    }

    let subcommand = &args[0];
    let subcommand_args = &args[1..];

    match subcommand.as_str() {
        "read" => output_read(subcommand_args),
        "tail" => output_tail(subcommand_args),
        "watch" => output_watch(subcommand_args),
        _ => {
            eprintln!(
                "{}Unknown output subcommand: '{}'",
                colors::red(),
                subcommand
            );
            std::process::exit(1);
        }
    }
}

pub fn handle_status(_args: &[String]) -> Result<()> {
    println!("{}System Status{}", colors::bold(), colors::reset());
    println!();

    let sessions = tmux_session::list_sessions()?;
    println!(
        "{}Active Sessions: {}{}",
        colors::cyan(),
        sessions.len(),
        colors::reset()
    );

    if sessions.is_empty() {
        println!(
            "  {}No active sessions{}",
            colors::yellow(),
            colors::reset()
        );
    } else {
        for session in &sessions {
            let status = if session.attached {
                format!("{}(attached){}", colors::green(), colors::reset())
            } else {
                format!("{}(detached){}", colors::yellow(), colors::reset())
            };
            println!(
                "  {}{}{} {}{} - {} windows {}",
                colors::blue(),
                session.id.0,
                colors::reset(),
                colors::bold(),
                session.name,
                colors::reset(),
                session.windows.len()
            );
            println!("  Status: {}", status);
        }
    }
    println!();

    let log_reader = LogReader::new();
    let logged_sessions = log_reader.list_session_logs()?;
    println!(
        "{}Available Logs: {}{}",
        colors::cyan(),
        logged_sessions.len(),
        colors::reset()
    );

    if logged_sessions.is_empty() {
        println!("  {}No log files{}", colors::yellow(), colors::reset());
    } else {
        for session_id in &logged_sessions {
            println!("  {}{}{}", colors::blue(), session_id.0, colors::reset());
        }
    }

    Ok(())
}

fn session_start(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session name required{}",
            colors::red(),
            colors::reset()
        );
        std::process::exit(1);
    }

    let name = &args[0];
    println!(
        "{}Creating session: {}{}{}",
        colors::cyan(),
        colors::bold(),
        name,
        colors::reset()
    );

    match tmux_session::new_session(name) {
        Ok(session) => {
            println!(
                "{}Session created successfully!{}",
                colors::green(),
                colors::reset()
            );
            println!(
                "  {}ID: {}{}",
                colors::blue(),
                session.id.0,
                colors::reset()
            );
            println!(
                "  {}Name: {}{}",
                colors::blue(),
                session.name,
                colors::reset()
            );
            println!(
                "  {}Windows: {}{}",
                colors::blue(),
                session.windows.len(),
                colors::reset()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{}Failed to create session: {}",
                colors::red(),
                e,
                colors::reset()
            );
            std::process::exit(1);
        }
    }
}

fn session_stop(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session id or name required{}",
            colors::red(),
            colors::reset()
        );
        std::process::exit(1);
    }

    let identifier = &args[0];
    println!(
        "{}Stopping session: {}{}{}",
        colors::cyan(),
        colors::bold(),
        identifier,
        colors::reset()
    );

    let sessions = tmux_session::list_sessions()?;
    let session = sessions
        .iter()
        .find(|s| s.id.0 == *identifier || s.name == *identifier);

    match session {
        Some(s) => {
            tmux_session::kill_session(&s.id)?;
            println!(
                "{}Session stopped successfully!{}",
                colors::green(),
                colors::reset()
            );
            Ok(())
        }
        None => {
            eprintln!(
                "{}Session not found: {}",
                colors::red(),
                identifier,
                colors::reset()
            );
            std::process::exit(1);
        }
    }
}

fn session_list(_args: &[String]) -> Result<()> {
    println!("{}Sessions{}", colors::bold(), colors::reset());
    println!();

    let sessions = tmux_session::list_sessions()?;

    if sessions.is_empty() {
        println!("{}No active sessions{}", colors::yellow(), colors::reset());
    } else {
        for session in &sessions {
            let status = if session.attached {
                format!("{}attached{}", colors::green(), colors::reset())
            } else {
                format!("{}detached{}", colors::yellow(), colors::reset())
            };
            println!("  {}{}{}", colors::blue(), session.id.0, colors::reset());
            println!("    Name: {}", colors::bold(), session.name);
            println!("    Status: {}", status);
            println!("    Windows: {}", session.windows.len());
            println!();
        }
    }

    Ok(())
}

fn session_attach(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session id or name required{}",
            colors::red(),
            colors::reset()
        );
        std::process::exit(1);
    }

    let identifier = &args[0];
    println!(
        "{}Attaching to session: {}{}{}",
        colors::cyan(),
        colors::bold(),
        identifier,
        colors::reset()
    );

    let sessions = tmux_session::list_sessions()?;
    let session = sessions
        .iter()
        .find(|s| s.id.0 == *identifier || s.name == *identifier);

    match session {
        Some(s) => {
            tmux_session::attach_session(&s.id)?;
            Ok(())
        }
        None => {
            eprintln!(
                "{}Session not found: {}",
                colors::red(),
                identifier,
                colors::reset()
            );
            std::process::exit(1);
        }
    }
}

fn session_detach(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session id or name required{}",
            colors::red(),
            colors::reset()
        );
        std::process::exit(1);
    }

    let identifier = &args[0];
    println!(
        "{}Detaching from session: {}{}{}",
        colors::cyan(),
        colors::bold(),
        identifier,
        colors::reset()
    );

    let sessions = tmux_session::list_sessions()?;
    let session = sessions
        .iter()
        .find(|s| s.id.0 == *identifier || s.name == *identifier);

    match session {
        Some(s) => {
            tmux_session::detach_session(&s.id)?;
            println!(
                "{}Detached from session{}",
                colors::green(),
                colors::reset()
            );
            Ok(())
        }
        None => {
            eprintln!(
                "{}Session not found: {}",
                colors::red(),
                identifier,
                colors::reset()
            );
            std::process::exit(1);
        }
    }
}

pub fn message_send(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        eprintln!(
            "{}Error: pane_id and message required{}",
            colors::red(),
            colors::reset()
        );
        eprintln!("  Usage: message send <pane_id> <message>");
        std::process::exit(1);
    }

    let pane_id = PaneId(args[0].clone());
    let message = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    println!(
        "{}Sending message to pane: {}{}",
        colors::cyan(),
        colors::bold(),
        pane_id.0,
        colors::reset()
    );
    println!("  Message: {}", message);

    // Hybrid approach: Write to file AND inject via tmux
    let base_path = PathBuf::from("/home/sam/code/swarm_test");
    let sender = MessageSender::new(base_path);

    // First write to file (for metadata/tracking)
    if let Err(e) = sender.send_prompt(&pane_id, &message) {
        eprintln!(
            "{}Failed to write prompt file: {}",
            colors::red(),
            e,
            colors::reset()
        );
    }

    // Then inject into tmux pane (for actual delivery)
    use swarm_test::tmux::pane;
    match pane::send_keys(&pane_id, &message) {
        Ok(_) => {
            println!(
                "{}Message sent successfully!{}",
                colors::green(),
                colors::reset()
            );
        }
        Err(e) => {
            eprintln!(
                "{}Failed to inject message: {}",
                colors::red(),
                e,
                colors::reset()
            );
            std::process::exit(1);
        }
    }
}

fn output_read(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session_id required{}",
            colors::red(),
            colors::reset()
        );
        std::process::exit(1);
    }

    let session_id = SessionId(args[0].clone());
    println!(
        "{}Reading output for session: {}{}",
        colors::cyan(),
        colors::bold(),
        session_id.0,
        colors::reset()
    );
    println!();

    let log_reader = LogReader::new();
    let output = log_reader.read_log(&session_id)?;

    if output.trim().is_empty() {
        println!("{}No output available{}", colors::yellow(), colors::reset());
    } else {
        print!("{}", output);
    }

    Ok(())
}

fn output_tail(args: &[String]) -> Result<()> {
    let session_id = match args.first() {
        Some(id) => SessionId(id.clone()),
        None => {
            eprintln!(
                "{}Error: session_id required{}",
                colors::red(),
                colors::reset()
            );
            std::process::exit(1);
        }
    };

    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);

    println!(
        "{}Tailing last {} lines for session: {}{}{}",
        colors::cyan(),
        n,
        colors::bold(),
        session_id.0,
        colors::reset()
    );
    println!();

    let log_reader = LogReader::new();
    let lines = log_reader.tail_log(&session_id, n)?;

    if lines.is_empty() {
        println!("{}No output available{}", colors::yellow(), colors::reset());
    } else {
        for line in &lines {
            println!("{}", line);
        }
    }

    Ok(())
}

fn output_watch(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!(
            "{}Error: session_id required{}",
            colors::red(),
            colors::reset()
        );
        std::process::exit(1);
    }

    let session_id = SessionId(args[0].clone());
    println!(
        "{}Watching output for session: {}{}{}",
        colors::cyan(),
        colors::bold(),
        session_id.0,
        colors::reset()
    );
    println!(
        "{}Press Ctrl+C to stop{}",
        colors::yellow(),
        colors::reset()
    );
    println!();

    let log_reader = LogReader::new();

    match log_reader.watch_log(&session_id, |line| {
        print!("{}", line);
        std::io::stdout().flush().unwrap();
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "{}Error watching log: {}",
                colors::red(),
                e,
                colors::reset()
            );
            std::process::exit(1);
        }
    }
}
