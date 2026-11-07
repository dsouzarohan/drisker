use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_drisker"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("the drisker binary should run")
}

#[test]
fn help_succeeds_on_stdout() {
    for flag in ["--help", "-h"] {
        let output = run(&[flag]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).expect("help should be UTF-8");
        assert!(help.contains("Usage: drisker"));
        assert!(help.contains("--help"));
        assert!(help.contains("--version"));
        assert!(help.contains("not implemented yet"));
    }
}

#[test]
fn version_matches_package_metadata() {
    for flag in ["--version", "-V"] {
        let output = run(&[flag]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let version = String::from_utf8(output.stdout).expect("version should be UTF-8");
        assert_eq!(
            version.trim(),
            concat!("drisker ", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn no_arguments_show_help_successfully() {
    let output = run(&[]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: drisker"));
}

#[test]
fn invalid_arguments_fail_on_stderr() {
    for argument in ["--unknown", "disk", "drivers"] {
        let output = run(&[argument]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(argument));
        assert!(error.contains("--help"));
    }
}
