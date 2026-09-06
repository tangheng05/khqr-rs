mod gen;
mod show;
mod watch;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "khqr",
    version,
    about = "Generate, read and watch KHQR payments"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

// clap parses each variant's args in place and cannot take a Box, so the
// size difference between subcommands is unavoidable.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Command {
    /// Build a payload, optionally writing a PNG or SVG alongside it.
    Gen(gen::Args),
    /// Print every field of a payload.
    Decode {
        /// The KHQR string.
        qr: String,
    },
    /// Check a payload's checksum. Exits non zero if it does not match.
    Verify {
        /// The KHQR string.
        qr: String,
    },
    /// Poll Bakong until a payment lands.
    Watch(watch::Args),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Gen(args) => gen::run(&args),
        Command::Decode { qr } => show::decode(&qr),
        Command::Verify { qr } => show::verify(&qr),
        Command::Watch(args) => watch::run(&args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("khqr: {error}");
            ExitCode::FAILURE
        }
    }
}
