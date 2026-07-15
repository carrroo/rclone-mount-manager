//! Dependency checker — verifies rclone and macFUSE installation.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DependencyCheck {
    pub rclone_installed: bool,
    pub macfuse_installed: bool,
    pub rclone_version: Option<String>,
}

/// Check whether rclone and macFUSE are installed on the system.
///
/// `rclone_path` is an already-resolved rclone binary path. If `None`, rclone
/// is treated as not installed.
pub fn check_dependencies(rclone_path: Option<&Path>) -> DependencyCheck {
    let rclone_installed = rclone_path.is_some();

    let rclone_version = rclone_path.and_then(|path| {
        Command::new(path)
            .arg("version")
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .and_then(|s| s.lines().next().map(|l| l.to_string()))
            })
    });

    let macfuse_paths = [
        PathBuf::from("/Library/Filesystems/macfuse.fs"),
        PathBuf::from("/usr/local/include/fuse/fuse.h"),
        PathBuf::from("/opt/homebrew/include/fuse/fuse.h"),
    ];
    let macfuse_installed = macfuse_paths.iter().any(|p| p.exists());

    DependencyCheck {
        rclone_installed,
        macfuse_installed,
        rclone_version,
    }
}
