mod disk;
mod output;

use clap::builder::{OsStringValueParser, TypedValueParser};
use clap::{CommandFactory, Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect disk contents without changing files.
    Disk {
        #[command(subcommand)]
        command: DiskCommand,
    },
}

#[derive(Debug, Subcommand)]
enum DiskCommand {
    /// Report observed logical bytes and entry counts beneath a directory.
    Scan {
        /// Directory to inspect. Links and Windows reparse points are skipped.
        #[arg(value_parser = OsStringValueParser::new().map(PathBuf::from))]
        path: PathBuf,
        /// Emit one JSON result, including scan diagnostics.
        #[arg(long)]
        json: bool,
    },
}

fn run(cli: Cli, stdout: &mut impl Write, stderr: &mut impl Write) -> io::Result<u8> {
    let Some(Command::Disk {
        command: DiskCommand::Scan { path, json },
    }) = cli.command
    else {
        Cli::command().write_help(&mut *stdout)?;
        writeln!(stdout)?;
        stdout.flush()?;
        return Ok(0);
    };
    let result = match std::env::current_dir() {
        Ok(working_directory) => disk::scan(&disk::ScanRequest {
            root: path.clone(),
            working_directory,
        }),
        Err(error) => Err(disk::ScanError::io(
            &path,
            disk::Operation::ValidateRoot,
            error,
        )),
    };
    output::render(&path, &result, json, stdout, stderr)?;
    Ok(output::exit_status(&result))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    match run(cli, &mut stdout, &mut stderr) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let _ = writeln!(stderr, "output error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn no_arguments_are_valid() {
        assert!(Cli::try_parse_from(["drisker"]).is_ok());
    }

    #[test]
    fn scan_requires_exactly_one_root() {
        for args in [
            vec!["drisker", "disk", "scan"],
            vec!["drisker", "disk", "scan", "a", "b"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
        assert!(Cli::try_parse_from(["drisker", "disk", "scan", ".", "--json"]).is_ok());
        assert!(Cli::try_parse_from(["drisker", "disk", "scan", "", "--json"]).is_ok());
    }

    #[test]
    fn unimplemented_commands_are_rejected() {
        let error = Cli::try_parse_from(["drisker", "drivers"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidSubcommand);
    }
}
