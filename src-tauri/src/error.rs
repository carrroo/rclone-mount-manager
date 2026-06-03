//! Application error types — structured errors with i18n-friendly codes.
//!
//! Each variant produces a string like `"error.rclone_not_found"` or
//! `"error.mount_failed:details"` that the frontend can route through
//! vue-i18n's `te()`/`t()` for localization.

/// Structured error type for the application.
/// Converts to user-facing error codes that the frontend can i18n-translate.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("error.rclone_not_found")]
    RcloneNotFound,

    #[error("error.read_conf_failed:{0}")]
    ConfReadFailed(#[from] std::io::Error),

    #[error("error.create_dir_failed:{0}")]
    CreateDirFailed(#[source] std::io::Error),

    #[error("error.mount_failed:{0}")]
    MountFailed(String),

    #[error("error.unmount_failed:{0}")]
    UnmountFailed(String),

    #[error("error.write_conf_failed:{0}")]
    WriteConfFailed(#[source] std::io::Error),

    #[error("error.write_conf_failed:invalid key: {0}")]
    InvalidKey(String),

    #[error("error.mount_failed:invalid extra arg: {0}")]
    InvalidArg(String),

    #[error("error.mount_failed:mount point must be under /Volumes/ or home directory")]
    PathNotAllowed,
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}
