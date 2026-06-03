//! Rclone.conf parser and updater.
//!
//! Reads the INI-style rclone.conf, extracts remote sections,
//! and updates whitelisted keys in-place.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Keys that may be updated in a remote section via the frontend.
pub const ALLOWED_CONFIG_KEYS: &[&str] = &["host", "user", "pass", "port"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub config_type: String,
    pub options: HashMap<String, String>,
}

/// Parse all remote sections from an rclone.conf file.
pub fn read_remotes(config_path: &Path) -> Result<Vec<RemoteConfig>, String> {
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::ConfReadFailed(e).to_string())?;

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

/// Update specific keys in a remote section of rclone.conf.
/// Only whitelisted keys are allowed (host, user, pass, port).
pub fn update_remote_config(
    config_path: &Path,
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
            return Err(AppError::InvalidKey(key.clone()).to_string());
        }
    }

    let content = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::ConfReadFailed(e).to_string())?;

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
    std::fs::write(config_path, new_content)
        .map_err(|e| AppError::WriteConfFailed(e).to_string())?;

    Ok(())
}
