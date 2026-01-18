//! Error types for ZManager operations.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for ZManager operations.
#[derive(Debug, Error)]
pub enum ZError {
    /// An I/O error occurred while accessing the filesystem.
    #[error("I/O error at '{path}': {message}")]
    Io {
        path: PathBuf,
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// Permission denied when accessing a path.
    #[error("Permission denied: '{path}'")]
    PermissionDenied { path: PathBuf },

    /// The specified path was not found.
    #[error("Path not found: '{path}'")]
    NotFound { path: PathBuf },

    /// The path is not a directory (when a directory was expected).
    #[error("Not a directory: '{path}'")]
    NotADirectory { path: PathBuf },

    /// The path is not a file (when a file was expected).
    #[error("Not a file: '{path}'")]
    NotAFile { path: PathBuf },

    /// A file or directory already exists at the target path.
    #[error("Already exists: '{path}'")]
    AlreadyExists { path: PathBuf },

    /// A directory is not empty (when trying to delete non-recursively).
    #[error("Directory not empty: '{path}'")]
    DirectoryNotEmpty { path: PathBuf },

    /// The path contains invalid characters or is malformed.
    #[error("Invalid path: '{path}' - {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    /// A symlink or junction target could not be resolved.
    #[error("Failed to resolve link at '{path}': {reason}")]
    LinkResolutionFailed { path: PathBuf, reason: String },

    /// An operation was cancelled by the user.
    #[error("Operation cancelled")]
    Cancelled,

    /// An invalid operation was attempted.
    #[error("Invalid operation '{operation}': {reason}")]
    InvalidOperation { operation: String, reason: String },

    /// A transfer operation failed.
    #[error("Transfer failed: {message}")]
    TransferFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration error.
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// A Windows-specific error occurred.
    #[error("Windows error (code {code}): {message}")]
    Windows { code: u32, message: String },

    /// An internal error that should not occur in normal operation.
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl ZError {
    /// Create an I/O error with path context.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        let path = path.into();
        let message = source.to_string();
        Self::Io {
            path,
            message,
            source,
        }
    }

    /// Create from an I/O error, inferring the appropriate variant.
    pub fn from_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        let path = path.into();
        match source.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound { path },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied { path },
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists { path },
            _ => Self::io(path, source),
        }
    }

    /// Check if this error represents a "not found" condition.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Check if this error represents a permission issue.
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied { .. })
    }

    /// Check if this error represents a cancellation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// Get the path associated with this error, if any.
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Io { path, .. }
            | Self::PermissionDenied { path }
            | Self::NotFound { path }
            | Self::NotADirectory { path }
            | Self::NotAFile { path }
            | Self::AlreadyExists { path }
            | Self::DirectoryNotEmpty { path }
            | Self::InvalidPath { path, .. }
            | Self::LinkResolutionFailed { path, .. } => Some(path),
            _ => None,
        }
    }
}

/// A type alias for `Result<T, ZError>`.
pub type ZResult<T> = Result<T, ZError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_creation() {
        let io_err = io::Error::other("test error");
        let err = ZError::io("C:\\test\\path", io_err);

        assert!(err.path().is_some());
        assert_eq!(err.path().unwrap().to_str().unwrap(), "C:\\test\\path");
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_from_io_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = ZError::from_io("C:\\missing", io_err);

        assert!(err.is_not_found());
        assert!(matches!(err, ZError::NotFound { .. }));
    }

    #[test]
    fn test_from_io_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = ZError::from_io("C:\\protected", io_err);

        assert!(err.is_permission_denied());
        assert!(matches!(err, ZError::PermissionDenied { .. }));
    }

    #[test]
    fn test_cancelled() {
        let err = ZError::Cancelled;
        assert!(err.is_cancelled());
        assert!(err.path().is_none());
    }

    #[test]
    fn test_error_display() {
        let err = ZError::NotFound {
            path: PathBuf::from("C:\\test"),
        };
        assert_eq!(err.to_string(), "Path not found: 'C:\\test'");

        let err = ZError::PermissionDenied {
            path: PathBuf::from("C:\\secret"),
        };
        assert_eq!(err.to_string(), "Permission denied: 'C:\\secret'");
    }
}
