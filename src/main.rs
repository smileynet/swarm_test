mod cli;

use cli::run;

fn main() {
    if let Err(e) = run() {
        eprintln!(
            "{}Error: {}{}",
            cli::main::colors::red(),
            e,
            cli::main::colors::reset()
        );
        std::process::exit(1);
    }
}
