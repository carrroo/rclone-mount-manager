//! Rclone.conf parser and updater.
//!
//! Reads the INI-style rclone.conf, extracts remote sections,
//! and updates whitelisted keys in-place.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Keys that may be updated in a remote section via the frontend.
pub const ALLOWED_CONFIG_KEYS: &[&str] = &["type", "host", "user", "pass", "port"];

/// Remote types that may be created or switched to via the frontend.
pub const ALLOWED_REMOTE_TYPES: &[&str] = &["sftp", "webdav", "http", "ftp"];

/// Validate a remote name: letters, digits, underscore, hyphen only.
/// rclone section names must not contain shell metacharacters, brackets,
/// whitespace, or newlines.
fn is_valid_remote_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Validate a remote type value.
fn is_valid_remote_type(config_type: &str) -> bool {
    ALLOWED_REMOTE_TYPES.contains(&config_type)
}

/// Validate a config value before writing it to rclone.conf.
/// Rejects values that would corrupt the INI structure (newlines/carriage returns).
fn is_valid_config_value(value: &str) -> bool {
    !value.contains('\n') && !value.contains('\r')
}

/// Cached regex for matching INI section headers like `[name]`.
static SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(.+?)\]").unwrap());

/// Cached regex for matching key=value lines.
static KV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(\w+)\s*=\s*(.+)").unwrap());

/// Cached regex for matching key=value lines with leading whitespace preserved.
static KV_FULL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(\w+)(\s*=\s*)(.*)").unwrap());

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

    let mut current_name: Option<String> = None;
    let mut current_options: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(caps) = SECTION_RE.captures(line) {
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
        } else if let Some(caps) = KV_RE.captures(line) {
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

    // Only allow whitelisted keys and validate values.
    let allowed: HashSet<&str> = ALLOWED_CONFIG_KEYS.iter().copied().collect();
    for (key, value) in &updates {
        if !allowed.contains(key.as_str()) {
            return Err(AppError::InvalidKey(key.clone()).to_string());
        }
        if !is_valid_config_value(value) {
            return Err(AppError::InvalidArg(format!(
                "value for '{}' contains invalid characters",
                key
            ))
            .to_string());
        }
        if key == "type" && !is_valid_remote_type(value) {
            return Err(AppError::InvalidArg(format!(
                "invalid remote type: {}",
                value
            ))
            .to_string());
        }
    }

    let content = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::ConfReadFailed(e).to_string())?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut in_target = false;
    let mut remaining: HashSet<String> = updates.keys().cloned().collect();
    let mut insert_pos: Option<usize> = None;

    for i in 0..lines.len() {
        let trimmed = lines[i].trim();
        if let Some(caps) = SECTION_RE.captures(trimmed) {
            if in_target {
                insert_pos = Some(i);
                break;
            }
            in_target = caps[1].to_string() == name;
        } else if in_target {
            if let Some(caps) = KV_FULL_RE.captures(&lines[i]) {
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

/// Add a new remote section to rclone.conf.
pub fn add_remote(
    config_path: &Path,
    rclone_path: &Path,
    name: &str,
    config_type: &str,
    options: HashMap<String, String>,
) -> Result<(), String> {
    // Validate section name and type to prevent config injection.
    if !is_valid_remote_name(name) {
        return Err(AppError::InvalidArg(format!(
            "invalid remote name: {}",
            name
        ))
        .to_string());
    }
    if !is_valid_remote_type(config_type) {
        return Err(AppError::InvalidArg(format!(
            "invalid remote type: {}",
            config_type
        ))
        .to_string());
    }

    let allowed: HashSet<&str> = ALLOWED_CONFIG_KEYS.iter().copied().collect();
    for (key, value) in &options {
        if !allowed.contains(key.as_str()) {
            return Err(AppError::InvalidKey(key.clone()).to_string());
        }
        if !is_valid_config_value(value) {
            return Err(AppError::InvalidArg(format!(
                "value for '{}' contains invalid characters",
                key
            ))
            .to_string());
        }
    }

    // Check if remote already exists
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::ConfReadFailed(e).to_string())?;

    let section_pattern = format!("[{}]", name);
    if content.contains(&section_pattern) {
        return Err(format!("Remote '{}' already exists", name));
    }

    // Obfuscate password if present
    let mut final_options = options.clone();
    if let Some(pass) = options.get("pass")
        && !pass.is_empty()
    {
        let obscured = obscure_password(rclone_path, pass)?;
        final_options.insert("pass".to_string(), obscured);
    }

    // Build new section. `type` is already written from config_type, so
    // skip it if the caller also included it in options.
    let mut new_section = format!("\n[{}]\ntype = {}\n", name, config_type);
    let mut keys: Vec<&String> = final_options.keys().collect();
    keys.sort();
    for key in keys {
        if key == "type" {
            continue;
        }
        let val = &final_options[key];
        if !val.is_empty() {
            new_section.push_str(&format!("{} = {}\n", key, val));
        }
    }

    // Append to file
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(config_path)
        .map_err(|e| AppError::WriteConfFailed(e).to_string())?;

    use std::io::Write;
    file.write_all(new_section.as_bytes())
        .map_err(|e| AppError::WriteConfFailed(e).to_string())?;

    Ok(())
}

/// Obfuscate a password using `rclone obscure`.
///
/// Reads the password from stdin so it does not appear in the process
/// command line.
fn obscure_password(rclone_path: &Path, password: &str) -> Result<String, String> {
    let mut child = std::process::Command::new(rclone_path)
        .arg("obscure")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run rclone obscure: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(password.as_bytes());
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to read rclone obscure output: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "rclone obscure failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
