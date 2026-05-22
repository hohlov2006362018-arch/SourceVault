use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("unrecognised archive format")]
    UnknownFormat,

    #[error("not a {expected} archive (magic mismatch)")]
    BadMagic { expected: &'static str },

    #[error("unsupported version {version} for {format} archive")]
    UnsupportedVersion { format: &'static str, version: u32 },

    #[error("malformed {format} archive: {reason}")]
    Malformed {
        format: &'static str,
        reason: String,
    },

    #[error("CRC mismatch on `{path}` (expected 0x{expected:08X}, got 0x{actual:08X})")]
    CrcMismatch {
        path: String,
        expected: u32,
        actual: u32,
    },

    #[error("entry `{0}` not found in archive")]
    EntryNotFound(String),

    #[error("entry `{0}` is a directory, not a file")]
    IsDirectory(String),

    #[error("data block file is missing or unreadable: {0}")]
    MissingDataPart(String),

    #[error("UTF-8 decoding failed: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("operation not supported by this format")]
    Unsupported,
}

impl Error {
    pub fn malformed(format: &'static str, reason: impl Into<String>) -> Self {
        Self::Malformed {
            format,
            reason: reason.into(),
        }
    }
}
