//! Mount and unmount operations — builds rclone mount commands
//! and executes diskutil for unmounting.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::AppError;
use super::detect::{find_rclone, is_path_allowed, is_safe_arg};

/// Build a nohup rclone mount command with logging.
pub fn build_mount_command(
    rclone_path: &PathBuf,
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

    let log_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("rclone-mount-manager");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("rclone-mount.log");

    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(format!(
            "nohup \"{}\" mount '{}' '{}' --vfs-cache-mode full --allow-non-empty {} > /dev/null 2>\"{}\" &",
            rclone_path.display(),
            remote_path.replace("'", "'\\''"),
            mount_point.replace("'", "'\\''"),
            extra_args.iter().map(|a| format!("'{}'", a.replace("'", "'\\''"))).collect::<Vec<_>>().join(" "),
            log_path.display(),
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    Ok(cmd)
}

/// Execute an rclone mount.
pub fn do_mount(remote_path: &str, mount_point: &str, extra_args: &[String]) -> Result<(), String> {
    let rclone_path = find_rclone().ok_or_else(|| AppError::RcloneNotFound.to_string())?;

    if !is_path_allowed(mount_point) {
        return Err(AppError::PathNotAllowed.to_string());
    }

    std::fs::create_dir_all(mount_point).map_err(|e| AppError::CreateDirFailed(e).to_string())?;

    let mut cmd = build_mount_command(&rclone_path, remote_path, mount_point, extra_args)?;

    let output = cmd
        .output()
        .map_err(|e| AppError::MountFailed(e.to_string()).to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::MountFailed(stderr.to_string()).to_string());
    }

    Ok(())
}

/// Unmount a filesystem using diskutil, with a timeout fallback.
///
/// If `diskutil unmount force` hangs (common when rclone crashed and
/// left a zombie FUSE mount), we kill the rclone process for that
/// mount point and retry once.
pub fn do_unmount(mount_point: &str) -> Result<(), String> {
    // Try unmount with a 10s timeout
    match unmount_with_timeout(mount_point, Duration::from_secs(10)) {
        Ok(output) if output.status.success() => return Ok(()),
        Ok(_output) => {
            // diskutil returned quickly but failed — try killing rclone first
            kill_rclone_for_mount(mount_point);
            // Retry once
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
            return Err(AppError::UnmountFailed(stderr.to_string()).to_string());
        }
        Err(_) => {
            // Timed out — kill rclone for this mount and retry
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
            return Err(AppError::UnmountFailed(stderr.to_string()).to_string());
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
fn kill_rclone_for_mount(mount_point: &str) {
    // Use pgrep + pkill approach: find rclone processes whose args contain the mount point
    let _ = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "pgrep -f 'rclone mount.*{}' | xargs kill -9 2>/dev/null || true",
            mount_point.replace("'", "'\\''"),
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Give the kernel a moment to clean up the FUSE connection
    std::thread::sleep(Duration::from_millis(500));
}
