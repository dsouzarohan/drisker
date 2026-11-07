use serde_json::Value;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

fn scan(path: &Path, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_drisker"));
    command.args([OsStr::new("disk"), OsStr::new("scan"), path.as_os_str()]);
    if json {
        command.arg("--json");
    }
    command.output().unwrap()
}

#[test]
fn text_and_json_scan_the_same_fixture() {
    let fixture = tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("nested space λ")).unwrap();
    std::fs::write(fixture.path().join("four"), b"1234").unwrap();
    std::fs::write(fixture.path().join("nested space λ/six"), b"123456").unwrap();
    let output = scan(fixture.path(), true);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "complete");
    assert_eq!(value["totals"]["logical_bytes"], 10);
    assert_eq!(value["totals"]["files"], 2);
    assert_eq!(value["totals"]["directories"], 1);
    let output = scan(fixture.path(), false);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Logical bytes: 10\n"));
    assert!(text.contains("Files: 2\n"));
    assert!(text.contains("Directories: 1\n"));
}

#[test]
fn root_failures_are_fatal_in_both_modes() {
    let fixture = tempdir().unwrap();
    let file = fixture.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    for (path, code) in [
        (fixture.path().join("missing"), "not_found"),
        (file, "invalid_root"),
        (Path::new("").to_owned(), "invalid_root"),
    ] {
        let output = scan(&path, true);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["status"], "failed");
        assert!(value["totals"].is_null());
        assert_eq!(value["fatal_error"]["code"], code);
        let output = scan(&path, false);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn scan_syntax_errors_are_usage_errors() {
    for args in [
        vec!["disk", "scan", "--json"],
        vec!["disk", "scan", ".", "."],
        vec!["disk", "scan", ".", "--unknown"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_drisker"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn relative_path_is_preserved_in_output() {
    let fixture = tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("root")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_drisker"))
        .current_dir(fixture.path())
        .args(["disk", "scan", "root", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["root"]["display"], "root");
    assert_eq!(value["root"]["lossy"], false);
}

#[cfg(unix)]
#[test]
fn binary_reports_depth_limit_as_partial_with_exit_three() {
    let fixture = tempdir().unwrap();
    let mut directory = fixture.path().to_owned();
    for _ in 0..257 {
        directory.push("d");
        std::fs::create_dir(&directory).unwrap();
    }
    std::fs::write(directory.join("beyond-limit"), b"x").unwrap();
    let output = scan(fixture.path(), true);
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["status"], "partial");
    assert_eq!(value["totals"]["directories"], 257);
    assert_eq!(value["totals"]["files"], 0);
    assert_eq!(value["error_count"], 1);
    assert_eq!(value["errors"][0]["code"], "depth_limit");
    let output = scan(fixture.path(), false);
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Status: partial"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("depth_limit"));
}
