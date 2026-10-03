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
        "install" => install::install(&optional_version(&mut args)?),
        "update" => update::update(&optional_version(&mut args)?),
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

fn optional_version(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let version = args
        .next()
        .unwrap_or_else(|| install::DEFAULT_SDK_VERSION.to_owned());
    if let Some(argument) = args.next() {
        return Err(format!("unexpected argument: `{argument}`"));
    }
    Ok(version)
}

fn print_help() {
    println!(
        "komeup

Usage:
\tkomeup install [SDK version]
\tkomeup update [SDK version]
\tkomeup uninstall
\tkomeup version"
    );
}
