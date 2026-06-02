use std::collections::HashMap;
use std::sync::atomic::Ordering;

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

    pub fn err(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

#[tauri::command]
pub fn get_all_mounts(
    manager: State<RcloneManager>,
    custom_mounts: Vec<MountItem>,
) -> ApiResponse<Vec<MountItem>> {
    match manager.get_all_mounts(custom_mounts) {
        Ok(items) => ApiResponse::ok(items),
        Err(e) => ApiResponse::err(e),
    }
}

#[tauri::command]
pub fn mount_remote(
    manager: State<RcloneManager>,
    remote_path: String,
    mount_point: String,
    extra_args: Vec<String>,
) -> ApiResponse<()> {
    match manager.mount(&remote_path, &mount_point, &extra_args) {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    }
}

#[tauri::command]
pub fn unmount_remote(
    manager: State<RcloneManager>,
    mount_point: String,
) -> ApiResponse<()> {
    match manager.unmount(&mount_point) {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    }
}

#[tauri::command]
pub fn update_remote_config(
    manager: State<RcloneManager>,
    name: String,
    updates: HashMap<String, String>,
) -> ApiResponse<()> {
    match manager.update_remote_config(&name, updates) {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e),
    }
}

#[tauri::command]
pub fn check_dependencies(manager: State<RcloneManager>) -> ApiResponse<DependencyCheck> {
    let deps = manager.check_dependencies();
    ApiResponse::ok(deps)
}

#[tauri::command]
pub fn start_auto_reconnect(
    manager: State<RcloneManager>,
    configs: Vec<MountItem>,
) {
    manager.start_reconnect_monitor(configs);
}

#[tauri::command]
pub fn get_language() -> String {
    if LANG_ZH.load(Ordering::SeqCst) {
        "zh".to_string()
    } else if LANG_EN.load(Ordering::SeqCst) {
        "en".to_string()
    } else {
        "system".to_string()
    }
}

#[tauri::command]
pub fn set_language(app: AppHandle, lang: String) {
    apply_lang(&app, &lang);
}
