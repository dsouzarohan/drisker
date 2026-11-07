use clap::{CommandFactory, Parser};
use std::io::{self, Write};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(
    after_help = "Early foundation: disk scanning and driver inspection are not implemented yet."
)]
struct Cli {}

fn main() -> io::Result<()> {
    Cli::parse();

    // Until inspection commands exist, an invocation without arguments shows help.
    let mut stdout = io::stdout().lock();
    Cli::command().write_help(&mut stdout)?;
    writeln!(stdout)
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
    fn unimplemented_commands_are_rejected() {
        for command in ["disk", "drivers"] {
            let error = Cli::try_parse_from(["drisker", command]).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::UnknownArgument);
        }
    }
}
