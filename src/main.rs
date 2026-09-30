mod install;
mod paths;
mod update;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("komeup: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        "install" => install::install(),
        "update" => update::update(),
        "uninstall" => install::uninstall(),

        "version" | "--version" | "-V" => {
            println!("komeup {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }

        _ => Err(format!("unknown command: `{command}`")),
    }
}

fn print_help() {
    println!(
        "komeup

Usage:
\tkomeup install
\tkomeup update
\tkomeup uninstall
\tkomeup version"
    );
}
