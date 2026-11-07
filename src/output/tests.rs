use super::*;
use crate::disk::{Operation, ScanError, ScanSummary, Totals};
use serde_json::{Value, json};

fn rendered(result: &Result<ScanSummary, ScanError>, json: bool) -> (String, String) {
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    render(Path::new("sample"), result, json, &mut stdout, &mut stderr).unwrap();
    (
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

#[test]
fn complete_example_matches_contract() {
    let result = Ok(ScanSummary {
        totals: Totals {
            logical_bytes: 10,
            files: 2,
            directories: 1,
            skipped_links: 1,
            skipped_special: 0,
        },
        ..ScanSummary::default()
    });
    let contract = include_str!("../../docs/disk-scan-contract.md").replace("\r\n", "\n");
    let example = contract
        .split("```json\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    let (stdout, stderr) = rendered(&result, true);
    assert!(stderr.is_empty());
    assert!(stdout.ends_with('\n'));
    assert_eq!(
        serde_json::from_str::<Value>(&stdout).unwrap(),
        serde_json::from_str::<Value>(example).unwrap()
    );
    let (stdout, stderr) = rendered(&result, false);
    let text_example = contract
        .split("```text\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    assert_eq!(stdout.trim_end(), text_example);
    assert!(stderr.is_empty());
    assert_eq!(exit_status(&result), 0);
}

#[test]
fn partial_results_put_diagnostics_in_the_selected_stream() {
    let error = ScanError::io(
        Path::new("sample/blocked"),
        Operation::ReadDirectory,
        io::ErrorKind::PermissionDenied.into(),
    );
    let result = Ok(ScanSummary {
        error_count: 103,
        errors: vec![error],
        ..ScanSummary::default()
    });
    let (stdout, stderr) = rendered(&result, true);
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["status"], "partial");
    assert_eq!(value["error_count"], 103);
    assert_eq!(value["errors_omitted"], 102);
    assert_eq!(value["errors"][0]["code"], "permission_denied");
    assert!(value["fatal_error"].is_null());
    assert!(stderr.is_empty());
    let (stdout, stderr) = rendered(&result, false);
    assert!(stdout.contains("Status: partial"));
    assert!(stdout.contains("Errors: 103 (102 omitted)"));
    assert!(stderr.contains("permission_denied"));
    assert_eq!(exit_status(&result), 3);
}

#[test]
fn fatal_results_have_no_totals_and_exit_one() {
    let result = Err(ScanError::io(
        Path::new("sample"),
        Operation::ValidateRoot,
        io::ErrorKind::NotFound.into(),
    ));
    let (stdout, stderr) = rendered(&result, true);
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["status"], "failed");
    for field in ["totals", "error_count", "errors_omitted"] {
        assert!(value[field].is_null());
    }
    assert_eq!(value["errors"], json!([]));
    assert_eq!(value["fatal_error"]["code"], "not_found");
    assert!(stderr.is_empty());
    let (stdout, stderr) = rendered(&result, false);
    assert!(stdout.is_empty());
    assert!(stderr.contains("not_found"));
    assert_eq!(exit_status(&result), 1);
}

#[test]
fn text_escapes_control_characters_and_json_round_trips_them() {
    let path = Path::new("root\n\u{1b}[31mλ");
    let result = Err(ScanError::io(
        path,
        Operation::ReadMetadata,
        io::Error::other("line\n\u{1b}[31mλ"),
    ));
    let (stdout, stderr) = rendered(&result, false);
    assert!(stdout.is_empty());
    assert_eq!(stderr.lines().count(), 1);
    assert!(!stderr.contains('\u{1b}'));
    assert!(stderr.contains("\\n\\u{1b}"));
    assert!(stderr.contains('λ'));
    let (stdout, _) = rendered(&result, true);
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        value["fatal_error"]["path"]["display"],
        path.to_str().unwrap()
    );
    assert_eq!(value["fatal_error"]["message"], "line\n\u{1b}[31mλ");
}

#[test]
fn json_preserves_u64_above_float_precision() {
    let result = Ok(ScanSummary {
        totals: Totals {
            logical_bytes: u64::MAX,
            ..Totals::default()
        },
        ..ScanSummary::default()
    });
    let (stdout, _) = rendered(&result, true);
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["totals"]["logical_bytes"].as_u64(), Some(u64::MAX));
}

#[cfg(any(unix, windows))]
#[test]
fn non_unicode_paths_are_explicitly_lossy() {
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(vec![b'x', 0xff]))
    };
    #[cfg(windows)]
    let path = {
        use std::os::windows::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_wide(&[b'x' as u16, 0xd800]))
    };
    let result = Ok(ScanSummary::default());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    render(&path, &result, true, &mut stdout, &mut stderr).unwrap();
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["root"]["lossy"], true);
    assert_eq!(value["root"]["display"], "x\u{fffd}");
}

struct BrokenWriter {
    fail_flush: bool,
}
impl Write for BrokenWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail_flush {
            Ok(buffer.len())
        } else {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn output_and_flush_failures_propagate_in_both_modes() {
    for json in [false, true] {
        for fail_flush in [false, true] {
            let result = Ok(ScanSummary::default());
            let error = render(
                Path::new("sample"),
                &result,
                json,
                &mut BrokenWriter { fail_flush },
                &mut Vec::new(),
            )
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        }
    }
}
