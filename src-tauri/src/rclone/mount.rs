//! Mount and unmount operations — builds rclone mount commands
//! and executes diskutil for unmounting.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::AppError;
use super::detect::{find_rclone, is_path_allowed, is_safe_arg};

/// Build an rclone mount command (not yet spawned).
///
/// Returns a `Command` configured with all arguments. The caller is
/// responsible for spawning it. Stdin is null; stdout/stderr should
/// be redirected by the caller (e.g. to a log file).
pub fn build_mount_command(
    rclone_path: &Path,
    remote_path: &str,
    mount_point: &str,
    extra_args: &[String],
) -> Result<Command, String> {
    if !is_safe_arg(remote_path) {
        return Err(AppError::InvalidArg("invalid remote path".to_string()).to_string());
    }
    if !is_safe_arg(mount_point) {
        return Err(AppError::InvalidArg("invalid mount point".to_string()).to_string());
    }
    for arg in extra_args {
        if !is_safe_arg(arg) {
            return Err(AppError::InvalidArg(arg.clone()).to_string());
        }
    }

    let mut cmd = Command::new(rclone_path);
    cmd.arg("mount")
        .arg(remote_path)
        .arg(mount_point)
        .arg("--vfs-cache-mode")
        .arg("full")
        .arg("--allow-non-empty")
        .stdin(Stdio::null());

    for arg in extra_args {
        cmd.arg(arg);
    }

    Ok(cmd)
}

/// Execute an rclone mount — spawns rclone as a background process.
pub fn do_mount(remote_path: &str, mount_point: &str, extra_args: &[String]) -> Result<(), String> {
    let rclone_path = find_rclone().ok_or_else(|| AppError::RcloneNotFound.to_string())?;

    if !is_path_allowed(mount_point) {
        return Err(AppError::PathNotAllowed.to_string());
    }

    std::fs::create_dir_all(mount_point).map_err(|e| AppError::CreateDirFailed(e).to_string())?;

    let log_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("rclone-mount-manager");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("rclone-mount.log");

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| AppError::MountFailed(e.to_string()).to_string())?;

    let mut cmd = build_mount_command(&rclone_path, remote_path, mount_point, extra_args)?;
    cmd.stdout(Stdio::from(log_file.try_clone().map_err(|e| AppError::MountFailed(e.to_string()).to_string())?));
    cmd.stderr(Stdio::from(log_file));

    cmd.spawn()
        .map_err(|e| AppError::MountFailed(e.to_string()).to_string())?;

    Ok(())
}

/// Unmount a filesystem using diskutil, with a timeout fallback.
///
/// If `diskutil unmount force` hangs (common when rclone crashed and
/// left a zombie FUSE mount), we kill the rclone process for that
/// mount point and retry once.
pub fn do_unmount(mount_point: &str) -> Result<(), String> {
    // Validate mount_point to prevent injection in kill_rclone_for_mount
    if !is_safe_arg(mount_point) {
        return Err(AppError::InvalidArg("invalid mount point".to_string()).to_string());
    }
    if !is_path_allowed(mount_point) {
        return Err(AppError::PathNotAllowed.to_string());
    }

    // Try unmount with a 10s timeout
    match unmount_with_timeout(mount_point, Duration::from_secs(10)) {
        Ok(output) if output.status.success() => return Ok(()),
        _ => {
            // Failed or timed out — kill rclone and retry once
            kill_rclone_for_mount(mount_point);
            let retry = Command::new("diskutil")
                .arg("unmount")
                .arg("force")
                .arg(mount_point)
                .output()
                .map_err(|e| AppError::UnmountFailed(e.to_string()).to_string())?;
            if retry.status.success() {
                return Ok(());
            }
            let stderr = String::from_utf8_lossy(&retry.stderr);
            Err(AppError::UnmountFailed(stderr.to_string()).to_string())
        }
    }
}

/// Run `diskutil unmount force` with a timeout using a child process.
fn unmount_with_timeout(mount_point: &str, timeout: Duration) -> Result<std::process::Output, ()> {
    let mut child = Command::new("diskutil")
        .arg("unmount")
        .arg("force")
        .arg(mount_point)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ())?;

    match child.try_wait() {
        Ok(Some(_status)) => {
            // Already finished
            let output = child.wait_with_output().map_err(|_| ())?;
            Ok(output)
        }
        Ok(None) => {
            // Still running — wait up to timeout
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        let output = child.wait_with_output().map_err(|_| ())?;
                        return Ok(output);
                    }
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            let _ = child.kill();
                            let _ = child.wait();
                            return Err(());
                        }
                        std::thread::sleep(Duration::from_millis(200));
                    }
                    Err(_) => {
                        let _ = child.kill();
                        return Err(());
                    }
                }
            }
        }
        Err(_) => {
            let _ = child.kill();
            Err(())
        }
    }
}

/// Find and kill the rclone process serving a specific mount point.
///
/// Uses `pgrep -f` to find rclone processes whose command-line args
/// contain the mount point, then sends SIGKILL via `kill -9`.
/// Runs pgrep and kill as separate commands (no shell interpolation)
/// to avoid shell injection.
fn kill_rclone_for_mount(mount_point: &str) {
    let pattern = format!("rclone mount.*{}", mount_point.replace("'", "'\\''"));

    // pgrep returns matching PIDs, one per line
    let output = match Command::new("pgrep")
        .arg("-f")
        .arg(&pattern)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return, // no matches or pgrep error
    };

    let pids = String::from_utf8_lossy(&output.stdout);
    for pid in pids.lines() {
        if let Ok(pid_num) = pid.trim().parse::<i32>() {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid_num.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    // Give the kernel a moment to clean up the FUSE connection
    std::thread::sleep(Duration::from_millis(500));
}
