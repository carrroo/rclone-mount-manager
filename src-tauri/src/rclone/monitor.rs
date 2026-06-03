//! Auto-reconnect monitor — background thread that polls mount status
//! every 30s and remounts any custom mounts that have dropped.

use std::process::{Command, Stdio};
use std::time::Duration;

use super::detect::{find_rclone, is_mount_point_active};
use super::mount::build_mount_command;
use super::MountItem;

/// Start a background thread that monitors custom mounts and auto-remounts
/// any that have been dropped.
pub fn start_reconnect_monitor(configs: Vec<MountItem>) {
    let rclone_path = match find_rclone() {
        Some(p) => p,
        None => return,
    };

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(30));

            let mount_output = Command::new("/sbin/mount")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();

            for config in &configs {
                if config.source != "custom" {
                    continue;
                }

                let mp = &config.mount_point;
                let is_mounted = is_mount_point_active(&mount_output, mp);

                if !is_mounted {
                    if let Ok(mut cmd) = build_mount_command(
                        &rclone_path,
                        &config.remote_path,
                        &config.mount_point,
                        &config.extra_args,
                    ) {
                        let _ = cmd
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn();
                    }
                }
            }
        }
    });
}
