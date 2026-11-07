use std::ffi::OsString;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

pub const MAX_DIRECTORY_DEPTH: usize = 256;
pub const MAX_ERROR_RECORDS: usize = 100;
pub const MAX_ERROR_MESSAGE_CHARS: usize = 1024;

pub struct ScanRequest {
    pub root: PathBuf,
    pub working_directory: PathBuf,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub logical_bytes: u64,
    pub files: u64,
    pub directories: u64,
    pub skipped_links: u64,
    pub skipped_special: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    ValidateRoot,
    ReadMetadata,
    ReadDirectory,
    Aggregate,
}

impl Operation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValidateRoot => "validate_root",
            Self::ReadMetadata => "read_metadata",
            Self::ReadDirectory => "read_directory",
            Self::Aggregate => "aggregate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidRoot,
    RootLink,
    NotFound,
    PermissionDenied,
    IoError,
    DepthLimit,
    CounterOverflow,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRoot => "invalid_root",
            Self::RootLink => "root_link",
            Self::NotFound => "not_found",
            Self::PermissionDenied => "permission_denied",
            Self::IoError => "io_error",
            Self::DepthLimit => "depth_limit",
            Self::CounterOverflow => "counter_overflow",
        }
    }
}

#[derive(Debug)]
pub struct ScanError {
    pub path: PathBuf,
    pub operation: Operation,
    pub code: ErrorCode,
    pub os_code: Option<i32>,
    pub message: String,
    pub message_truncated: bool,
}

impl ScanError {
    fn new(path: &Path, operation: Operation, code: ErrorCode, message: &str) -> Self {
        let mut chars = message.chars();
        let message = chars.by_ref().take(MAX_ERROR_MESSAGE_CHARS).collect();
        Self {
            path: path.to_owned(),
            operation,
            code,
            os_code: None,
            message,
            message_truncated: chars.next().is_some(),
        }
    }

    pub fn io(path: &Path, operation: Operation, error: io::Error) -> Self {
        let code = match error.kind() {
            io::ErrorKind::NotFound => ErrorCode::NotFound,
            io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::IoError,
        };
        let mut record = Self::new(path, operation, code, &error.to_string());
        record.os_code = error.raw_os_error();
        record
    }
}

#[derive(Debug, Default)]
pub struct ScanSummary {
    pub totals: Totals,
    pub error_count: u64,
    pub errors: Vec<ScanError>,
}

impl ScanSummary {
    fn record(&mut self, error: ScanError) -> Result<(), ScanError> {
        add(&mut self.error_count, 1, &error.path)?;
        if self.errors.len() < MAX_ERROR_RECORDS {
            self.errors.push(error);
        }
        Ok(())
    }

    pub fn errors_omitted(&self) -> u64 {
        self.error_count - self.errors.len() as u64
    }
}

fn add(counter: &mut u64, amount: u64, path: &Path) -> Result<(), ScanError> {
    *counter = counter.checked_add(amount).ok_or_else(|| {
        ScanError::new(
            path,
            Operation::Aggregate,
            ErrorCode::CounterOverflow,
            "scan counter exceeds u64",
        )
    })?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum EntryKind {
    File(u64),
    Directory,
    Link,
    Special,
}

fn classify(metadata: &Metadata) -> EntryKind {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return EntryKind::Link;
        }
    }
    let kind = metadata.file_type();
    if kind.is_symlink() {
        EntryKind::Link
    } else if kind.is_file() {
        EntryKind::File(metadata.len())
    } else if kind.is_dir() {
        EntryKind::Directory
    } else {
        EntryKind::Special
    }
}

fn read_directory(path: &Path) -> io::Result<impl Iterator<Item = io::Result<OsString>> + use<>> {
    Ok(fs::read_dir(path)?.map(|entry| entry.map(|entry| entry.file_name())))
}

pub fn scan(request: &ScanRequest) -> Result<ScanSummary, ScanError> {
    scan_with(
        request,
        |path| fs::symlink_metadata(path).map(|m| classify(&m)),
        read_directory,
    )
}

struct Frame<I> {
    physical: PathBuf,
    reported: PathBuf,
    depth: usize,
    entries: I,
}

fn scan_with<I>(
    request: &ScanRequest,
    mut metadata: impl FnMut(&Path) -> io::Result<EntryKind>,
    mut read: impl FnMut(&Path) -> io::Result<I>,
) -> Result<ScanSummary, ScanError>
where
    I: Iterator<Item = io::Result<OsString>>,
{
    let root = &request.root;
    if root.as_os_str().is_empty() {
        return Err(ScanError::new(
            root,
            Operation::ValidateRoot,
            ErrorCode::InvalidRoot,
            "root must be a directory",
        ));
    }
    // Remove trailing separators before classifying the final component.
    let normalized: PathBuf = root.components().collect();
    let physical = request.working_directory.join(normalized);
    let kind = metadata(&physical).map_err(|e| ScanError::io(root, Operation::ValidateRoot, e))?;
    match kind {
        EntryKind::Directory => {}
        EntryKind::Link => {
            return Err(ScanError::new(
                root,
                Operation::ValidateRoot,
                ErrorCode::RootLink,
                "root is a link or reparse point",
            ));
        }
        _ => {
            return Err(ScanError::new(
                root,
                Operation::ValidateRoot,
                ErrorCode::InvalidRoot,
                "root must be a directory",
            ));
        }
    }
    let entries = read(&physical).map_err(|e| ScanError::io(root, Operation::ReadDirectory, e))?;
    let mut stack = vec![Frame {
        physical,
        reported: root.clone(),
        depth: 0,
        entries,
    }];
    let mut summary = ScanSummary::default();
    while let Some(frame) = stack.last_mut() {
        let Some(entry) = frame.entries.next() else {
            stack.pop();
            continue;
        };
        let name = match entry {
            Ok(name) => name,
            Err(e) => {
                summary.record(ScanError::io(&frame.reported, Operation::ReadDirectory, e))?;
                continue;
            }
        };
        let physical = frame.physical.join(&name);
        let reported = frame.reported.join(&name);
        let depth = frame.depth + 1;
        let kind = match metadata(&physical) {
            Ok(kind) => kind,
            Err(e) => {
                summary.record(ScanError::io(&reported, Operation::ReadMetadata, e))?;
                continue;
            }
        };
        match kind {
            EntryKind::File(bytes) => {
                add(&mut summary.totals.files, 1, &reported)?;
                add(&mut summary.totals.logical_bytes, bytes, &reported)?;
            }
            EntryKind::Link => add(&mut summary.totals.skipped_links, 1, &reported)?,
            EntryKind::Special => add(&mut summary.totals.skipped_special, 1, &reported)?,
            EntryKind::Directory => {
                add(&mut summary.totals.directories, 1, &reported)?;
                if depth > MAX_DIRECTORY_DEPTH {
                    summary.record(ScanError::new(
                        &reported,
                        Operation::ReadDirectory,
                        ErrorCode::DepthLimit,
                        "directory depth exceeds 256",
                    ))?;
                    continue;
                }
                match read(&physical) {
                    Ok(entries) => stack.push(Frame {
                        physical,
                        reported,
                        depth,
                        entries,
                    }),
                    Err(e) => {
                        summary.record(ScanError::io(&reported, Operation::ReadDirectory, e))?
                    }
                }
            }
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests;
