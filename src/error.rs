use std::io;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MultiBootError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device is not removable: {0}")]
    NotRemovable(String),

    #[error("Device is mounted: {0}")]
    DeviceMounted(String),

    #[error("Insufficient space on device (need {needed}, have {available})")]
    InsufficientSpace { needed: u64, available: u64 },

    #[error("ISO file not found: {0}")]
    IsoNotFound(String),

    #[error("Invalid ISO file: {0}")]
    InvalidIso(String),

    #[error("Partition error: {0}")]
    PartitionError(String),

    #[error("Filesystem error: {0}")]
    FilesystemError(String),

    #[error("Bootloader installation failed: {0}")]
    BootloaderError(String),

    #[error("Command execution failed: {cmd} - {message}")]
    CommandFailed { cmd: String, message: String },

    #[error("Permission denied - please run as root")]
    PermissionDenied,

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MultiBootError>;

// Implement From<io::Error> manually since io::Error doesn't implement Clone
impl From<io::Error> for MultiBootError {
    fn from(err: io::Error) -> Self {
        MultiBootError::Io(err.to_string())
    }
}
