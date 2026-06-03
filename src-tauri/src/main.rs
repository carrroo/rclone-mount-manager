use tauri::{
    image::Image,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, RunEvent,
};

use rclone_mount_manager::commands::*;
use rclone_mount_manager::lang::*;
use rclone_mount_manager::rclone::RcloneManager;

fn load_tray_icon() -> Option<Image<'static>> {
    let tray_32 = include_bytes!("../icons/tray-32.png");
    if let Ok(img) = image::load_from_memory(tray_32) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        return Some(Image::new_owned(rgba.into_raw(), width, height));
    }
    None
}

fn main() {
    init_language_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(RcloneManager::new())
        .invoke_handler(tauri::generate_handler![
            get_all_mounts,
            mount_remote,
            unmount_remote,
            update_remote_config,
            start_auto_reconnect,
            check_dependencies,
            get_language,
            set_language,
        ])
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "about" => {
                let _ = app.emit("show-about", ());
            }
            "lang_system" | "lang_zh" | "lang_en" => {
                let lang = match event.id.as_ref() {
                    "lang_zh" => "zh",
                    "lang_en" => "en",
                    _ => "system",
                };
                apply_lang(app, lang);
            }
            _ => {}
        })
        .setup(|app| {
            let cur = current_lang();

            // Shared menu items (used by both app menu and tray menu)
            let show_i = MenuItem::with_id(app, "show", t("menu.open", cur), true, None::<&str>)?;
            let quit_i = PredefinedMenuItem::quit(app, Some(&t("menu.quit", cur)))?;

            // Language submenu
            let lang_system = MenuItem::with_id(
                app, "lang_system", build_lang_text("system", cur), true, None::<&str>,
            )?;
            let lang_zh = MenuItem::with_id(
                app, "lang_zh", build_lang_text("zh", cur), true, None::<&str>,
            )?;
            let lang_en = MenuItem::with_id(
                app, "lang_en", build_lang_text("en", cur), true, None::<&str>,
            )?;
            let lang_menu = Submenu::with_id_and_items(
                app, "language", t("menu.language", cur), true,
                &[&lang_system, &lang_zh, &lang_en],
            )?;

            // App submenu
            let about_i = PredefinedMenuItem::about(
                app,
                Some("About Rclone Mount Manager"),
                Some(AboutMetadata {
                    name: Some("Rclone Mount Manager".to_string()),
                    version: Some("0.1.0".to_string()),
                    ..Default::default()
                }),
            )?;
            let app_menu = Submenu::with_id_and_items(
                app, "app", t("menu.app_title", cur), true,
                &[
                    &about_i,
                    &PredefinedMenuItem::separator(app)?,
                    &show_i,
                    &PredefinedMenuItem::separator(app)?,
                    &quit_i,
                ],
            )?;

            // Edit submenu — required for ⌘X/C/V/A shortcuts to work in webview inputs
            let edit_menu = Submenu::with_id_and_items(
                app, "edit", t("menu.edit", cur), true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;

            let menu = Menu::with_items(app, &[&app_menu, &edit_menu, &lang_menu])?;
            app.set_menu(menu)?;

            // Tray menu — reuse the same show_i and quit_i instances
            let tray_menu = Menu::with_items(app, &[
                &show_i,
                &PredefinedMenuItem::separator(app)?,
                &quit_i,
            ])?;

            let icon = load_tray_icon()
                .unwrap_or_else(|| app.default_window_icon().unwrap().clone());

            TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("构建失败")
        .run(|app_handle, event| {
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}
