use super::commands;
use std::env;
use std::process;
use swarm_test::Result;

pub fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        commands::print_usage();
        process::exit(0);
    }

    let command = &args[1];
    let command_args = &args[2..];

    match command.as_str() {
        "session" => commands::handle_session(command_args),
        "message" => commands::handle_message(command_args),
        "output" => commands::handle_output(command_args),
        "status" => commands::handle_status(command_args),
        "help" => {
            commands::print_usage();
            Ok(())
        }
        _ => {
            eprintln!("{}Unknown command: '{}'", colors::red(), command);
            eprintln!("{}Use 'help' for usage information", colors::reset());
            process::exit(1);
        }
    }
}

pub mod colors {
    pub fn red() -> &'static str {
        "\x1b[31m"
    }

    pub fn green() -> &'static str {
        "\x1b[32m"
    }

    pub fn yellow() -> &'static str {
        "\x1b[33m"
    }

    pub fn blue() -> &'static str {
        "\x1b[34m"
    }

    pub fn cyan() -> &'static str {
        "\x1b[36m"
    }

    pub fn bold() -> &'static str {
        "\x1b[1m"
    }

    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
}
