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

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub use config::{read_remotes, update_remote_config, add_remote, RemoteConfig, ALLOWED_CONFIG_KEYS};
pub use deps::{check_dependencies, DependencyCheck};
pub(crate) use detect::detect_mounted_remotes;
pub use detect::{find_rclone, is_path_allowed, is_safe_arg, is_mount_point_active};
pub use mount::{build_mount_command, do_mount, do_unmount};

use crate::error::AppError;
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
    prefs_path: PathBuf,
    /// Cancellation flag for the reconnect monitor thread.
    monitor_cancel: Arc<AtomicBool>,
    /// Handle to the running monitor thread (if any).
    ///
    /// `std::sync::Mutex` is intentional: these methods are synchronous and the
    /// guard is never held across an `.await`. If this code is ever made async,
    /// switch to `tokio::sync::Mutex`.
    monitor_handle: Mutex<Option<JoinHandle<()>>>,
    /// Cached path to the rclone binary. Resolved lazily by
    /// `resolve_rclone_path()`.
    rclone_path: Mutex<Option<PathBuf>>,
}

/// Persisted mount preferences (user-configured paths and display order).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MountPrefs {
    /// Display order of remote names.
    #[serde(default)]
    pub order: Vec<String>,
    /// Map from remote name → { remote_path, mount_point }
    pub paths: HashMap<String, PrefPaths>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefPaths {
    pub remote_path: String,
    pub mount_point: String,
}

impl Default for RcloneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RcloneManager {
    pub fn new() -> Self {
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config/rclone/rclone.conf");

        let prefs_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config/rclone-mount-manager/mount_prefs.json");

        Self {
            config_path,
            prefs_path,
            monitor_cancel: Arc::new(AtomicBool::new(false)),
            monitor_handle: Mutex::new(None),
            rclone_path: Mutex::new(None),
        }
    }

    /// Resolve the path to the rclone binary, using a cached value if available.
    fn resolve_rclone_path(&self) -> Result<PathBuf, String> {
        // Fast path: already cached.
        if let Ok(guard) = self.rclone_path.lock()
            && let Some(path) = guard.clone()
        {
            return Ok(path);
        }

        // Slow path: search the filesystem.
        let path = find_rclone().ok_or_else(|| AppError::RcloneNotFound.to_string())?;

        // Cache it for subsequent calls.
        if let Ok(mut guard) = self.rclone_path.lock() {
            *guard = Some(path.clone());
        }

        Ok(path)
    }

    /// Load persisted mount preferences.
    fn load_prefs(&self) -> MountPrefs {
        std::fs::read_to_string(&self.prefs_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save mount preferences to disk.
    fn save_prefs(&self, prefs: &MountPrefs) -> Result<(), String> {
        if let Some(parent) = self.prefs_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
        std::fs::write(&self.prefs_path, json).map_err(|e| e.to_string())
    }

    /// Save path preferences for a specific remote.
    pub fn save_mount_pref(&self, name: &str, remote_path: &str, mount_point: &str) -> Result<(), String> {
        let mut prefs = self.load_prefs();
        prefs.paths.insert(name.to_string(), PrefPaths {
            remote_path: remote_path.to_string(),
            mount_point: mount_point.to_string(),
        });
        self.save_prefs(&prefs)
    }

    /// Save display order of remotes.
    pub fn save_mount_order(&self, order: Vec<String>) -> Result<(), String> {
        let mut prefs = self.load_prefs();
        prefs.order = order;
        self.save_prefs(&prefs)
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn get_all_mounts(&self, custom_mounts: Vec<MountItem>) -> Result<Vec<MountItem>, String> {
        let remotes = read_remotes(&self.config_path)?;
        let mount_output = detect::get_mount_output();
        let detected = detect_mounted_remotes(&mount_output);
        let prefs = self.load_prefs();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        let mut items = Vec::new();

        for remote in remotes {
            let default_mount_point = home.join("mnt").join(&remote.name).to_string_lossy().to_string();
            let (mounted, actual_remote_path, actual_mount_point) =
                if let Some(d) = detected.get(&remote.name) {
                    (true, d.remote_path.clone(), d.mount_point.clone())
                } else if let Some(p) = prefs.paths.get(&remote.name) {
                    // Use saved user preferences
                    (false, p.remote_path.clone(), p.mount_point.clone())
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

        // Sort by saved order if available
        if !prefs.order.is_empty() {
            items.sort_by(|a, b| {
                let ia = prefs.order.iter().position(|n| n == &a.name);
                let ib = prefs.order.iter().position(|n| n == &b.name);
                match (ia, ib) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            });
        }

        Ok(items)
    }

    pub fn mount(&self, remote_path: &str, mount_point: &str, extra_args: &[String]) -> Result<(), String> {
        let rclone_path = self.resolve_rclone_path()?;
        do_mount(&rclone_path, remote_path, mount_point, extra_args)
    }

    pub fn unmount(&self, mount_point: &str) -> Result<(), String> {
        do_unmount(mount_point)
    }

    pub fn update_remote_config(&self, name: &str, updates: std::collections::HashMap<String, String>) -> Result<(), String> {
        update_remote_config(&self.config_path, name, updates)
    }

    pub fn add_remote_config(&self, name: &str, config_type: &str, options: std::collections::HashMap<String, String>) -> Result<(), String> {
        let rclone_path = self.resolve_rclone_path()?;
        add_remote(&self.config_path, &rclone_path, name, config_type, options)
    }

    pub fn check_dependencies(&self) -> DependencyCheck {
        let rclone_path = self.resolve_rclone_path().ok();
        check_dependencies(rclone_path.as_deref())
    }

    /// Start the auto-reconnect monitor. Stops any existing monitor first.
    pub fn start_reconnect_monitor(&self, configs: Vec<MountItem>) {
        // Stop existing monitor if running
        self.stop_reconnect_monitor();

        // If rclone is not available, there is nothing to monitor.
        let rclone_path = match self.resolve_rclone_path() {
            Ok(p) => p,
            Err(_) => return,
        };

        // Reset cancellation flag
        self.monitor_cancel.store(false, Ordering::Relaxed);

        // Start new monitor
        let handle = monitor::start_reconnect_monitor(rclone_path, configs, self.monitor_cancel.clone());

        if let Ok(mut guard) = self.monitor_handle.lock() {
            *guard = handle;
        }
    }

    /// Stop the auto-reconnect monitor and wait for it to finish.
    pub fn stop_reconnect_monitor(&self) {
        self.monitor_cancel.store(true, Ordering::Relaxed);

        if let Ok(mut guard) = self.monitor_handle.lock()
            && let Some(handle) = guard.take()
        {
            let _ = handle.join();
        }
    }
}
