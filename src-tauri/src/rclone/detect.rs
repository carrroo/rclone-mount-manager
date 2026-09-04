//! Mount detection — find rclone binary, validate paths/args,
//! parse /sbin/mount output to detect active rclone filesystems.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

/// Internal struct for a detected mount from /sbin/mount output.
pub(crate) struct DetectedMount {
    pub remote_path: String,
    pub mount_point: String,
}

/// Cached regex for parsing `/sbin/mount` output lines.
/// Matches lines like: `remote:path on /Volumes/mp (fstype, ...)`
/// capturing the device, mount point, and filesystem type.
static MOUNT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^:]+:.*?)\s+on\s+(.+?)\s+\((\w+)").unwrap());

/// Cached regex for extracting mount points from `/sbin/mount` output.
static MOUNT_POINT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"on\s+(.+?)\s+\(").unwrap());

/// Find the rclone binary on the system.
pub fn find_rclone() -> Option<PathBuf> {
    let candidates = [
        "/opt/homebrew/bin/rclone",
        "/usr/local/bin/rclone",
        "/usr/bin/rclone",
    ];
    for path in &candidates {
        if PathBuf::from(path).exists() {
            return Some(PathBuf::from(path));
        }
    }
    if let Ok(output) = std::process::Command::new("/usr/bin/which")
        .arg("rclone")
        .output()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

/// Validate that a path is under an allowed parent directory.
///
/// Tries `canonicalize` first (resolves symlinks, confirms existence).
/// Falls back to prefix matching when the path doesn't exist (e.g. a
/// stale mount point during unmount). If the home directory cannot be
/// determined, only `/Volumes/` is allowed in the fallback.
pub fn is_path_allowed(path: &str) -> bool {
    if let Ok(p) = PathBuf::from(path).canonicalize() {
        if p.starts_with("/Volumes/") {
            return true;
        }
        return dirs::home_dir()
            .map(|h| !h.as_os_str().is_empty() && p.starts_with(h))
            .unwrap_or(false);
    }
    // Fallback: string-based prefix check for paths that may not exist.
    if path.starts_with("/Volumes/") {
        return true;
    }
    dirs::home_dir()
        .map(|h| {
            let home_str = h.to_string_lossy().to_string();
            !home_str.is_empty() && path.starts_with(&home_str)
        })
        .unwrap_or(false)
}

/// Validate a string contains no shell metacharacters.
pub fn is_safe_arg(s: &str) -> bool {
    !s.contains(&['"', '\'', '`', '$', '\\', ';', '|', '&', '(', ')', '{', '}', '<', '>', '!', '#', '\n', '\r'][..])
}

/// Read the current mount table output from /sbin/mount.
pub fn get_mount_output() -> String {
    std::process::Command::new("/sbin/mount")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Parse the mount output to detect rclone-remoted filesystems.
/// Returns a map from remote name → (remote_path, mount_point).
///
/// Only FUSE mounts (macFUSE reports `osxfuse`, newer versions `macfuse`)
/// are considered — without this filter an NFS mount like
/// `server:/export on /mnt (nfs, ...)` whose server name happens to match
/// a remote name would be falsely reported as mounted.
pub(crate) fn detect_mounted_remotes(mount_output: &str) -> HashMap<String, DetectedMount> {
    let mut detected: HashMap<String, DetectedMount> = HashMap::new();

    for line in mount_output.lines() {
        if let Some(caps) = MOUNT_RE.captures(line) {
            let fstype = &caps[3];
            if !matches!(fstype, "osxfuse" | "macfuse" | "fusefs") {
                continue;
            }
            let full_remote = caps[1].trim();
            let mount_point = caps[2].trim();
            if let Some(colon_pos) = full_remote.find(':') {
                let remote_name = full_remote[..colon_pos].to_string();
                detected.insert(
                    remote_name,
                    DetectedMount {
                        remote_path: full_remote.to_string(),
                        mount_point: mount_point.to_string(),
                    },
                );
            }
        }
    }

    detected
}

/// Check if a specific mount point is currently active.
pub fn is_mount_point_active(mount_output: &str, mount_point: &str) -> bool {
    mount_output.lines().any(|line| {
        MOUNT_POINT_RE
            .captures(line)
            .map(|c| c[1].trim() == mount_point)
            .unwrap_or(false)
    })
}
