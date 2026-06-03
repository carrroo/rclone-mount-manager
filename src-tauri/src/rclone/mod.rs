//! Rclone integration module — manages rclone.conf, mount/unmount,
//! dependency checking, and auto-reconnect.
//!
//! Sub-modules:
//! - `config`  — Parse and update rclone.conf
//! - `detect`  — Find rclone binary, validate paths, detect mounted filesystems
//! - `mount`   — Build and execute mount/unmount commands
//! - `deps`    — Check rclone and macFUSE installation
//! - `monitor` — Background thread that auto-remounts dropped custom mounts

mod config;
mod deps;
mod detect;
mod mount;
mod monitor;

use std::path::PathBuf;

pub use config::{read_remotes, update_remote_config, RemoteConfig, ALLOWED_CONFIG_KEYS};
pub use deps::{check_dependencies, DependencyCheck};
pub(crate) use detect::detect_mounted_remotes;
pub use detect::{find_rclone, is_path_allowed, is_safe_arg, is_mount_point_active};
pub use mount::{build_mount_command, do_mount, do_unmount};
pub use monitor::start_reconnect_monitor;

use serde::{Deserialize, Serialize};

/// A single mount item (from rclone.conf or custom).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountItem {
    pub id: String,
    pub name: String,
    pub remote_path: String,
    pub mount_point: String,
    pub source: String,
    pub mounted: bool,
    pub config_type: String,
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub pass: String,
    #[serde(default)]
    pub port: String,
}

/// Central manager for rclone operations.
pub struct RcloneManager {
    config_path: PathBuf,
}

impl RcloneManager {
    pub fn new() -> Self {
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config/rclone/rclone.conf");

        Self { config_path }
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn get_all_mounts(&self, custom_mounts: Vec<MountItem>) -> Result<Vec<MountItem>, String> {
        let remotes = read_remotes(&self.config_path)?;
        let mount_output = detect::get_mount_output();

        let detected = detect_mounted_remotes(&mount_output);
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        let mut items = Vec::new();

        for remote in remotes {
            let default_mount_point = home.join("mnt").join(&remote.name).to_string_lossy().to_string();
            let (mounted, actual_remote_path, actual_mount_point) =
                if let Some(d) = detected.get(&remote.name) {
                    (true, d.remote_path.clone(), d.mount_point.clone())
                } else {
                    (false, format!("{}:/", remote.name), default_mount_point)
                };

            items.push(MountItem {
                id: format!("config:{}", remote.name),
                name: remote.name.clone(),
                remote_path: actual_remote_path,
                mount_point: actual_mount_point,
                source: "config".to_string(),
                mounted,
                config_type: remote.config_type,
                extra_args: Vec::new(),
                host: remote.options.get("host").cloned().unwrap_or_default(),
                user: remote.options.get("user").cloned().unwrap_or_default(),
                pass: remote.options.get("pass").cloned().unwrap_or_default(),
                port: remote.options.get("port").cloned().unwrap_or_default(),
            });
        }

        for mut custom in custom_mounts {
            let mp = &custom.mount_point;
            custom.mounted = is_mount_point_active(&mount_output, mp);
            items.push(custom);
        }

        Ok(items)
    }

    pub fn mount(&self, remote_path: &str, mount_point: &str, extra_args: &[String]) -> Result<(), String> {
        do_mount(remote_path, mount_point, extra_args)
    }

    pub fn unmount(&self, mount_point: &str) -> Result<(), String> {
        do_unmount(mount_point)
    }

    pub fn update_remote_config(&self, name: &str, updates: std::collections::HashMap<String, String>) -> Result<(), String> {
        update_remote_config(&self.config_path, name, updates)
    }

    pub fn check_dependencies(&self) -> DependencyCheck {
        check_dependencies()
    }

    pub fn start_reconnect_monitor(&self, configs: Vec<MountItem>) {
        start_reconnect_monitor(configs);
    }
}
