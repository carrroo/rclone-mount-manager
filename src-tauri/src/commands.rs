//! Tauri command handlers — thin wrappers that delegate to the rclone module
//! and wrap results in ApiResponse<T>.
//!
//! All commands that perform blocking I/O or subprocess calls are `async` and
//! run the blocking work inside `tauri::async_runtime::spawn_blocking`, so the
//! Tauri main thread stays responsive.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::lang::*;
use crate::rclone::{DependencyCheck, MountItem, RcloneManager};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// Convenience alias for async command return types.
///
/// Returning `Result` is required by Tauri for async commands that hold
/// `State`; the inner `ApiResponse` preserves the existing frontend contract.
pub type CmdResult<T> = Result<ApiResponse<T>, String>;

#[tauri::command]
pub async fn get_all_mounts(
    manager: State<'_, Arc<RcloneManager>>,
    custom_mounts: Vec<MountItem>,
) -> CmdResult<Vec<MountItem>> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.get_all_mounts(custom_mounts)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(items) => ApiResponse::ok(items),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn mount_remote(
    manager: State<'_, Arc<RcloneManager>>,
    remote_path: String,
    mount_point: String,
    extra_args: Vec<String>,
) -> CmdResult<()> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.mount(&remote_path, &mount_point, &extra_args)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn unmount_remote(
    manager: State<'_, Arc<RcloneManager>>,
    mount_point: String,
) -> CmdResult<()> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.unmount(&mount_point)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn update_remote_config(
    manager: State<'_, Arc<RcloneManager>>,
    name: String,
    updates: HashMap<String, String>,
) -> CmdResult<()> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.update_remote_config(&name, updates)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn add_remote_config(
    manager: State<'_, Arc<RcloneManager>>,
    name: String,
    config_type: String,
    options: HashMap<String, String>,
) -> CmdResult<()> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.add_remote_config(&name, &config_type, options)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn save_mount_pref(
    manager: State<'_, Arc<RcloneManager>>,
    name: String,
    remote_path: String,
    mount_point: String,
) -> CmdResult<()> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.save_mount_pref(&name, &remote_path, &mount_point)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn save_mount_order(
    manager: State<'_, Arc<RcloneManager>>,
    order: Vec<String>,
) -> CmdResult<()> {
    let manager = manager.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        manager.save_mount_order(order)
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(match result {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    })
}

#[tauri::command]
pub async fn check_dependencies(
    manager: State<'_, Arc<RcloneManager>>,
) -> CmdResult<DependencyCheck> {
    let manager = manager.inner().clone();
    let deps = tauri::async_runtime::spawn_blocking(move || {
        manager.check_dependencies()
    }).await.map_err(|e| format!("background task failed: {}", e))?;

    Ok(ApiResponse::ok(deps))
}

#[tauri::command]
pub fn get_language() -> String {
    current_lang().to_string()
}

#[tauri::command]
pub fn set_language(app: AppHandle, lang: String) {
    apply_lang(&app, &lang);
}
