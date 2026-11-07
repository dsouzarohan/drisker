use crate::disk::{self, ScanError, ScanSummary, Totals};
use serde::Serialize;
use std::borrow::Cow;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize)]
struct DisplayPath<'a> {
    display: Cow<'a, str>,
    lossy: bool,
}

impl<'a> From<&'a Path> for DisplayPath<'a> {
    fn from(path: &'a Path) -> Self {
        Self {
            display: path.to_string_lossy(),
            lossy: path.to_str().is_none(),
        }
    }
}

#[derive(Serialize)]
struct Diagnostic<'a> {
    path: DisplayPath<'a>,
    operation: &'static str,
    code: &'static str,
    os_code: Option<i32>,
    message: &'a str,
    message_truncated: bool,
}

impl<'a> From<&'a ScanError> for Diagnostic<'a> {
    fn from(error: &'a ScanError) -> Self {
        Self {
            path: error.path.as_path().into(),
            operation: error.operation.as_str(),
            code: error.code.as_str(),
            os_code: error.os_code,
            message: &error.message,
            message_truncated: error.message_truncated,
        }
    }
}

#[derive(Serialize)]
struct Policy {
    size: &'static str,
    hard_links: &'static str,
    links_and_reparse_points: &'static str,
    special_entries: &'static str,
    volume_boundaries: &'static str,
    consistency: &'static str,
    max_directory_depth: usize,
    max_error_records: usize,
    max_error_message_chars: usize,
}

#[derive(Serialize)]
struct JsonTotals {
    logical_bytes: u64,
    files: u64,
    directories: u64,
    skipped_links: u64,
    skipped_special: u64,
}

impl From<&Totals> for JsonTotals {
    fn from(totals: &Totals) -> Self {
        Self {
            logical_bytes: totals.logical_bytes,
            files: totals.files,
            directories: totals.directories,
            skipped_links: totals.skipped_links,
            skipped_special: totals.skipped_special,
        }
    }
}

#[derive(Serialize)]
struct Envelope<'a> {
    schema_version: u8,
    root: DisplayPath<'a>,
    status: &'static str,
    policy: Policy,
    totals: Option<JsonTotals>,
    error_count: Option<u64>,
    errors_omitted: Option<u64>,
    errors: Vec<Diagnostic<'a>>,
    fatal_error: Option<Diagnostic<'a>>,
}

fn status(result: &Result<ScanSummary, ScanError>) -> &'static str {
    match result {
        Ok(summary) if summary.error_count == 0 => "complete",
        Ok(_) => "partial",
        Err(_) => "failed",
    }
}

pub fn exit_status(result: &Result<ScanSummary, ScanError>) -> u8 {
    match result {
        Ok(summary) if summary.error_count == 0 => 0,
        Ok(_) => 3,
        Err(_) => 1,
    }
}

fn escaped(value: &str) -> String {
    let mut escaped = String::new();
    for c in value.chars() {
        if c.is_control() {
            escaped.extend(c.escape_default());
        } else {
            escaped.push(c);
        }
    }
    escaped
}

fn write_diagnostic(writer: &mut impl Write, error: &ScanError) -> io::Result<()> {
    writeln!(
        writer,
        "{}: {} ({}): {}{}",
        escaped(&error.path.to_string_lossy()),
        error.operation.as_str(),
        error.code.as_str(),
        escaped(&error.message),
        if error.message_truncated {
            " [message truncated]"
        } else {
            ""
        }
    )
}

pub fn render(
    root: &Path,
    result: &Result<ScanSummary, ScanError>,
    json: bool,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<()> {
    if json {
        let summary = result.as_ref().ok();
        let envelope = Envelope {
            schema_version: 1,
            root: root.into(),
            status: status(result),
            policy: Policy {
                size: "logical",
                hard_links: "per_name",
                links_and_reparse_points: "skip",
                special_entries: "skip",
                volume_boundaries: "cross_ordinary_mounts",
                consistency: "live_observations",
                max_directory_depth: disk::MAX_DIRECTORY_DEPTH,
                max_error_records: disk::MAX_ERROR_RECORDS,
                max_error_message_chars: disk::MAX_ERROR_MESSAGE_CHARS,
            },
            totals: summary.map(|s| (&s.totals).into()),
            error_count: summary.map(|s| s.error_count),
            errors_omitted: summary.map(ScanSummary::errors_omitted),
            errors: summary
                .map(|s| s.errors.iter().map(Diagnostic::from).collect())
                .unwrap_or_default(),
            fatal_error: result.as_ref().err().map(Diagnostic::from),
        };
        serde_json::to_writer(&mut *stdout, &envelope)?;
        writeln!(stdout)?;
    } else {
        match result {
            Err(error) => write_diagnostic(stderr, error)?,
            Ok(summary) => {
                let totals = &summary.totals;
                writeln!(stdout, "Root: {}", escaped(&root.to_string_lossy()))?;
                writeln!(stdout, "Status: {}", status(result))?;
                writeln!(stdout, "Logical bytes: {}", totals.logical_bytes)?;
                writeln!(stdout, "Files: {}", totals.files)?;
                writeln!(stdout, "Directories: {}", totals.directories)?;
                writeln!(
                    stdout,
                    "Skipped links/reparse points: {}",
                    totals.skipped_links
                )?;
                writeln!(
                    stdout,
                    "Skipped special entries: {}",
                    totals.skipped_special
                )?;
                writeln!(
                    stdout,
                    "Accounting: logical bytes per file name; hard links counted per name."
                )?;
                writeln!(
                    stdout,
                    "Boundaries: links/reparse points skipped; ordinary mounts traversed."
                )?;
                writeln!(
                    stdout,
                    "Limits: live observations, not a snapshot or allocated/reclaimable storage."
                )?;
                if summary.error_count > 0 {
                    writeln!(
                        stdout,
                        "Errors: {} ({} omitted)",
                        summary.error_count,
                        summary.errors_omitted()
                    )?;
                }
                for error in &summary.errors {
                    write_diagnostic(stderr, error)?;
                }
            }
        }
    }
    stdout.flush()?;
    stderr.flush()
}

#[cfg(test)]
mod tests;
