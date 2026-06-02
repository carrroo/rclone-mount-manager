use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use regex::Regex;
use serde::{Deserialize, Serialize};

const ALLOWED_CONFIG_KEYS: &[&str] = &["host", "user", "pass", "port"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub config_type: String,
    pub options: HashMap<String, String>,
}

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

#[derive(Debug, Clone, Serialize)]
pub struct DependencyCheck {
    pub rclone_installed: bool,
    pub macfuse_installed: bool,
    pub rclone_version: Option<String>,
}

fn find_rclone() -> Option<PathBuf> {
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
    if let Ok(output) = Command::new("/usr/bin/which").arg("rclone").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

/// Validate that a path is under an allowed parent directory.
fn is_path_allowed(path: &str) -> bool {
    let Ok(p) = PathBuf::from(path).canonicalize() else { return false };
    p.starts_with("/Volumes/") || p.starts_with(dirs::home_dir().unwrap_or_default())
}

/// Validate a string contains no shell metacharacters.
fn is_safe_arg(s: &str) -> bool {
    !s.contains(&['"', '\'', '`', '$', '\\', ';', '|', '&', '(', ')', '{', '}', '<', '>', '!', '#', '\n', '\r'][..])
}

fn build_mount_command(rclone_path: &PathBuf, remote_path: &str, mount_point: &str, extra_args: &[String]) -> Result<Command, String> {
    if !is_safe_arg(remote_path) {
        return Err("error.mount_failed:invalid remote path".to_string());
    }
    if !is_safe_arg(mount_point) {
        return Err("error.mount_failed:invalid mount point".to_string());
    }
    for arg in extra_args {
        if !is_safe_arg(arg) {
            return Err(format!("error.mount_failed:invalid extra arg: {}", arg));
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

    fn read_remotes(&self) -> Result<Vec<RemoteConfig>, String> {
        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("error.read_conf_failed:{}", e))?;

        let mut remotes = Vec::new();
        let section_re = Regex::new(r"\[(.+?)\]").unwrap();
        let kv_re = Regex::new(r"^\s*(\w+)\s*=\s*(.+)").unwrap();

        let mut current_name: Option<String> = None;
        let mut current_options: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(caps) = section_re.captures(line) {
                if let Some(name) = current_name.take() {
                    let config_type = current_options
                        .remove("type")
                        .unwrap_or_else(|| "unknown".to_string());
                    remotes.push(RemoteConfig {
                        name,
                        config_type,
                        options: std::mem::take(&mut current_options),
                    });
                }
                current_name = Some(caps[1].to_string());
            } else if let Some(caps) = kv_re.captures(line) {
                current_options.insert(caps[1].to_string(), caps[2].to_string());
            }
        }

        if let Some(name) = current_name {
            let config_type = current_options
                .remove("type")
                .unwrap_or_else(|| "unknown".to_string());
            remotes.push(RemoteConfig {
                name,
                config_type,
                options: current_options,
            });
        }

        Ok(remotes)
    }

    pub fn get_all_mounts(&self, custom_mounts: Vec<MountItem>) -> Result<Vec<MountItem>, String> {
        let remotes = self.read_remotes()?;
        let mount_output = Command::new("/sbin/mount")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let mount_re = Regex::new(r"^([^:]+:.*?)\s+on\s+(.+?)\s+\(").unwrap();
        struct DetectedMount {
            remote_path: String,
            mount_point: String,
        }
        let mut detected: HashMap<String, DetectedMount> = HashMap::new();
        for line in mount_output.lines() {
            if let Some(caps) = mount_re.captures(line) {
                let full_remote = caps[1].trim();
                let mount_point = caps[2].trim();
                if let Some(colon_pos) = full_remote.find(':') {
                    let remote_name = full_remote[..colon_pos].to_string();
                    detected.insert(remote_name, DetectedMount {
                        remote_path: full_remote.to_string(),
                        mount_point: mount_point.to_string(),
                    });
                }
            }
        }

        let mut items = Vec::new();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

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
            custom.mounted = mount_output.lines().any(|line| {
                let re = Regex::new(r"on\s+(.+?)\s+\(").unwrap();
                re.captures(line)
                    .map(|c| c[1].trim() == mp)
                    .unwrap_or(false)
            });
            items.push(custom);
        }

        Ok(items)
    }

    pub fn mount(&self, remote_path: &str, mount_point: &str, extra_args: &[String]) -> Result<(), String> {
        let rclone_path = find_rclone()
            .ok_or_else(|| "error.rclone_not_found".to_string())?;

        if !is_path_allowed(mount_point) {
            return Err("error.mount_failed:mount point must be under /Volumes/ or home directory".to_string());
        }

        std::fs::create_dir_all(mount_point)
            .map_err(|e| format!("error.create_dir_failed:{}", e))?;

        let mut cmd = build_mount_command(&rclone_path, remote_path, mount_point, extra_args)?;

        let output = cmd.output()
            .map_err(|e| format!("error.mount_failed:{}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("error.mount_failed:{}", stderr));
        }

        Ok(())
    }

    pub fn update_remote_config(
        &self,
        name: &str,
        updates: HashMap<String, String>,
    ) -> Result<(), String> {
        if updates.is_empty() {
            return Ok(());
        }

        // Only allow whitelisted keys
        let allowed: HashSet<&str> = ALLOWED_CONFIG_KEYS.iter().copied().collect();
        for key in updates.keys() {
            if !allowed.contains(key.as_str()) {
                return Err(format!("error.write_conf_failed:invalid key: {}", key));
            }
        }

        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("error.read_conf_failed:{}", e))?;

        let section_re = Regex::new(r"\[(.+?)\]").unwrap();
        let kv_re = Regex::new(r"^(\s*)(\w+)(\s*=\s*)(.*)").unwrap();

        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut in_target = false;
        let mut remaining: HashSet<String> = updates.keys().cloned().collect();
        let mut insert_pos: Option<usize> = None;

        for i in 0..lines.len() {
            let trimmed = lines[i].trim();
            if let Some(caps) = section_re.captures(trimmed) {
                if in_target {
                    insert_pos = Some(i);
                    break;
                }
                in_target = caps[1].to_string() == name;
            } else if in_target {
                if let Some(caps) = kv_re.captures(&lines[i]) {
                    let key = caps[2].to_string();
                    if let Some(new_val) = updates.get(&key) {
                        lines[i] = format!("{}{}{}{}", &caps[1], &caps[2], &caps[3], new_val);
                        remaining.remove(&key);
                    }
                }
            }
        }

        let insert_at = insert_pos.unwrap_or(lines.len());
        let remaining_sorted: Vec<String> = {
            let mut v: Vec<String> = remaining.into_iter().collect();
            v.sort();
            v
        };
        for (offset, key) in remaining_sorted.iter().enumerate() {
            if let Some(val) = updates.get(key) {
                lines.insert(insert_at + offset, format!("{} = {}", key, val));
            }
        }

        let new_content = lines.join("\n") + "\n";
        std::fs::write(&self.config_path, new_content)
            .map_err(|e| format!("error.write_conf_failed:{}", e))?;

        Ok(())
    }

    pub fn unmount(&self, mount_point: &str) -> Result<(), String> {
        let output = Command::new("diskutil")
            .arg("unmount")
            .arg("force")
            .arg(mount_point)
            .output()
            .map_err(|e| format!("error.unmount_failed:{}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("error.unmount_failed:{}", stderr));
        }

        Ok(())
    }

    pub fn check_dependencies(&self) -> DependencyCheck {
        let rclone_path = find_rclone();
        let rclone_installed = rclone_path.is_some();

        let rclone_version = if let Some(path) = rclone_path {
            Command::new(path)
                .arg("version")
                .output()
                .ok()
                .and_then(|o| {
                    String::from_utf8(o.stdout)
                        .ok()
                        .and_then(|s| s.lines().next().map(|l| l.to_string()))
                })
        } else {
            None
        };

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

    pub fn start_reconnect_monitor(
        &self,
        configs: Vec<MountItem>,
    ) {
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
                    let is_mounted = mount_output.lines().any(|line| {
                        let re = Regex::new(r"on\s+(.+?)\s+\(").unwrap();
                        re.captures(line)
                            .map(|c| c[1].trim() == mp)
                            .unwrap_or(false)
                    });

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
}
