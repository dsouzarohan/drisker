use super::*;
use tempfile::tempdir;

fn request(root: &Path) -> ScanRequest {
    ScanRequest {
        root: root.to_owned(),
        working_directory: std::env::current_dir().unwrap(),
    }
}

fn names(values: &[&str]) -> std::vec::IntoIter<io::Result<OsString>> {
    values
        .iter()
        .map(|name| Ok(OsString::from(name)))
        .collect::<Vec<_>>()
        .into_iter()
}

#[test]
fn empty_and_nested_unicode_directories_and_hard_links() {
    let fixture = tempdir().unwrap();
    let root = fixture.path();
    assert_eq!(scan(&request(root)).unwrap().totals, Totals::default());
    fs::create_dir(root.join("nested space λ")).unwrap();
    fs::write(root.join("four"), b"1234").unwrap();
    fs::write(root.join("nested space λ/six"), b"123456").unwrap();
    fs::hard_link(root.join("four"), root.join("same file")).unwrap();
    let summary = scan(&request(root)).unwrap();
    assert_eq!(
        summary.totals,
        Totals {
            logical_bytes: 14,
            files: 3,
            directories: 1,
            ..Totals::default()
        }
    );
    assert_eq!(summary.error_count, 0);
}

#[test]
fn relative_root_uses_captured_working_directory() {
    let fixture = tempdir().unwrap();
    fs::create_dir(fixture.path().join("root")).unwrap();
    fs::write(fixture.path().join("root/file"), b"abc").unwrap();
    let summary = scan(&ScanRequest {
        root: "root".into(),
        working_directory: fixture.path().to_owned(),
    })
    .unwrap();
    assert_eq!(summary.totals.logical_bytes, 3);
}

#[test]
fn invalid_roots_have_no_summary() {
    let fixture = tempdir().unwrap();
    let file = fixture.path().join("file");
    fs::write(&file, b"abc").unwrap();
    for (root, code) in [
        (PathBuf::new(), ErrorCode::InvalidRoot),
        (file, ErrorCode::InvalidRoot),
        (fixture.path().join("missing"), ErrorCode::NotFound),
    ] {
        let error = scan(&request(&root)).unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.operation, Operation::ValidateRoot);
        assert_eq!(error.path, root);
    }
}

#[test]
fn root_enumeration_failure_is_fatal() {
    let error = scan_with(
        &request(Path::new("root")),
        |_| Ok(EntryKind::Directory),
        |_| -> io::Result<std::vec::IntoIter<io::Result<OsString>>> {
            Err(io::ErrorKind::PermissionDenied.into())
        },
    )
    .unwrap_err();
    assert_eq!(error.operation, Operation::ReadDirectory);
    assert_eq!(error.code, ErrorCode::PermissionDenied);
}

#[test]
fn recoverable_failures_keep_siblings_and_classified_directories() {
    let root = Path::new("root");
    let summary = scan_with(
        &request(root),
        |path| match path.file_name().unwrap().to_str().unwrap() {
            "missing" => Err(io::ErrorKind::NotFound.into()),
            "good" => Ok(EntryKind::File(7)),
            "socket" => Ok(EntryKind::Special),
            _ => Ok(EntryKind::Directory),
        },
        |path| {
            if path.file_name().unwrap() == "blocked" {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            let mut entries = vec![Err(io::ErrorKind::Other.into())];
            entries.extend(names(&["blocked", "missing", "good", "socket"]));
            Ok(entries.into_iter())
        },
    )
    .unwrap();
    assert_eq!(
        summary.totals,
        Totals {
            logical_bytes: 7,
            files: 1,
            directories: 1,
            skipped_special: 1,
            ..Totals::default()
        }
    );
    assert_eq!(summary.error_count, 3);
    assert_eq!(summary.errors[0].path, root);
    assert_eq!(summary.errors[0].operation, Operation::ReadDirectory);
    assert_eq!(summary.errors[1].code, ErrorCode::PermissionDenied);
    assert_eq!(summary.errors[2].path, root.join("missing"));
    assert_eq!(summary.errors[2].operation, Operation::ReadMetadata);
    assert_eq!(summary.errors[2].code, ErrorCode::NotFound);
}

#[test]
fn diagnostic_cap_does_not_stop_scanning() {
    let summary = scan_with(
        &request(Path::new("root")),
        |path| {
            if path.file_name().unwrap() == "good" {
                Ok(EntryKind::File(9))
            } else {
                Ok(EntryKind::Directory)
            }
        },
        |_| {
            let mut entries = (0..103)
                .map(|_| Err(io::ErrorKind::PermissionDenied.into()))
                .collect::<Vec<_>>();
            entries.push(Ok("good".into()));
            Ok(entries.into_iter())
        },
    )
    .unwrap();
    assert_eq!(summary.error_count, 103);
    assert_eq!(summary.errors.len(), 100);
    assert_eq!(summary.errors_omitted(), 3);
    assert_eq!(summary.totals.logical_bytes, 9);
}

#[test]
fn depth_limit_counts_directory_but_does_not_open_it() {
    let mut opened = 0;
    let summary = scan_with(
        &request(Path::new("root")),
        |path| {
            if path.file_name().unwrap() == "file" {
                Ok(EntryKind::File(1))
            } else {
                Ok(EntryKind::Directory)
            }
        },
        |_| {
            opened += 1;
            Ok(names(&["directory", "file"]))
        },
    )
    .unwrap();
    assert_eq!(opened, MAX_DIRECTORY_DEPTH + 1);
    assert_eq!(summary.totals.directories, 257);
    assert_eq!(summary.totals.files, 257);
    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.errors[0].code, ErrorCode::DepthLimit);
}

#[test]
fn byte_overflow_discards_scan_summary() {
    let error = scan_with(
        &request(Path::new("root")),
        |path| match path.file_name().unwrap().to_str().unwrap() {
            "huge" => Ok(EntryKind::File(u64::MAX)),
            "extra" => Ok(EntryKind::File(1)),
            _ => Ok(EntryKind::Directory),
        },
        |_| Ok(names(&["huge", "extra"])),
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::CounterOverflow);
    assert_eq!(error.operation, Operation::Aggregate);
    assert_eq!(error.path, Path::new("root/extra"));
}

#[test]
fn counter_and_error_count_overflow_are_checked() {
    let path = Path::new("root/file");
    let mut count = u64::MAX;
    assert_eq!(
        add(&mut count, 1, path).unwrap_err().code,
        ErrorCode::CounterOverflow
    );
    assert_eq!(count, u64::MAX);
    let mut summary = ScanSummary {
        error_count: u64::MAX,
        ..ScanSummary::default()
    };
    let error = ScanError::io(
        path,
        Operation::ReadMetadata,
        io::ErrorKind::NotFound.into(),
    );
    assert_eq!(
        summary.record(error).unwrap_err().code,
        ErrorCode::CounterOverflow
    );
}

#[test]
fn error_messages_are_bounded_on_unicode_boundaries() {
    let message = "λ".repeat(MAX_ERROR_MESSAGE_CHARS + 1);
    let error = ScanError::io(
        Path::new("root"),
        Operation::ReadMetadata,
        io::Error::other(message),
    );
    assert_eq!(error.message.chars().count(), MAX_ERROR_MESSAGE_CHARS);
    assert!(error.message_truncated);
    let error = ScanError::io(
        Path::new("root"),
        Operation::ReadMetadata,
        io::Error::other("λ".repeat(MAX_ERROR_MESSAGE_CHARS)),
    );
    assert!(!error.message_truncated);
}

#[test]
fn os_error_code_is_preserved() {
    let error = ScanError::io(
        Path::new("root"),
        Operation::ReadMetadata,
        io::Error::from_raw_os_error(2),
    );
    assert_eq!(error.os_code, Some(2));
    assert_eq!(error.code, ErrorCode::NotFound);
}

#[cfg(unix)]
#[test]
fn links_are_skipped_and_link_roots_rejected_even_with_trailing_separator() {
    use std::os::unix::fs::symlink;
    let fixture = tempdir().unwrap();
    let root = fixture.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("file"), b"1234").unwrap();
    fs::create_dir(root.join("subdirectory")).unwrap();
    fs::write(root.join("subdirectory/six"), b"123456").unwrap();
    symlink("file", root.join("file-link")).unwrap();
    symlink("subdirectory", root.join("directory-link")).unwrap();
    symlink("missing", root.join("dangling-link")).unwrap();
    symlink(".", root.join("cycle")).unwrap();
    let summary = scan(&request(&root)).unwrap();
    assert_eq!(summary.totals.logical_bytes, 10);
    assert_eq!(summary.totals.files, 2);
    assert_eq!(summary.totals.directories, 1);
    assert_eq!(summary.totals.skipped_links, 4);
    assert_eq!(summary.error_count, 0);
    for path in [root.join("directory-link"), root.join("directory-link/")] {
        assert_eq!(scan(&request(&path)).unwrap_err().code, ErrorCode::RootLink);
    }
}

#[cfg(unix)]
#[test]
fn unix_socket_is_skipped_without_opening_it() {
    use std::os::unix::net::UnixListener;
    let fixture = tempdir().unwrap();
    let _listener = UnixListener::bind(fixture.path().join("socket")).unwrap();
    let summary = scan(&request(fixture.path())).unwrap();
    assert_eq!(summary.totals.skipped_special, 1);
    assert_eq!(summary.totals.logical_bytes, 0);
    assert_eq!(summary.error_count, 0);
}

#[cfg(windows)]
#[test]
fn windows_junction_is_a_skipped_reparse_point_and_invalid_root() {
    let fixture = tempdir().unwrap();
    fs::create_dir(fixture.path().join("root")).unwrap();
    fs::create_dir(fixture.path().join("target")).unwrap();
    fs::write(fixture.path().join("target/hidden"), b"do not count").unwrap();
    let output = std::process::Command::new("cmd")
        .current_dir(fixture.path())
        .args(["/C", "mklink", "/J", "root\\junction", "target"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let junction = fixture.path().join("root/junction");
    assert_eq!(
        classify(&fs::symlink_metadata(&junction).unwrap()),
        EntryKind::Link
    );
    let summary = scan(&request(&fixture.path().join("root"))).unwrap();
    assert_eq!(summary.totals.skipped_links, 1);
    assert_eq!(summary.totals.files, 0);
    assert_eq!(summary.totals.logical_bytes, 0);
    assert_eq!(summary.error_count, 0);
    assert_eq!(
        scan(&request(&junction)).unwrap_err().code,
        ErrorCode::RootLink
    );
    assert_eq!(
        scan(&request(&junction.join(""))).unwrap_err().code,
        ErrorCode::RootLink
    );
}
