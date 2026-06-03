//! Native menu text updater — refreshes menu item labels after a language change.

use tauri::AppHandle;

use super::translate::{build_lang_text, t};

/// Update all language-related menu item text after a language change.
pub fn update_lang_menu_text(app: &AppHandle, current: &str) {
    let menu = match app.menu() {
        Some(m) => m,
        None => return,
    };

    // Update app submenu title and items
    if let Some(item) = menu.get("app") {
        if let Some(sub) = item.as_submenu() {
            let _ = sub.set_text(t("menu.app_title", current));
            if let Some(show_item) = sub.get("show") {
                if let Some(m) = show_item.as_menuitem() {
                    let _ = m.set_text(t("menu.open", current));
                }
            }
            if let Some(quit_item) = sub.get("quit") {
                if let Some(m) = quit_item.as_predefined_menuitem() {
                    let _ = m.set_text(t("menu.quit", current));
                }
            }
        }
    }

    // Update edit submenu title
    if let Some(item) = menu.get("edit") {
        if let Some(sub) = item.as_submenu() {
            let _ = sub.set_text(t("menu.edit", current));
        }
    }

    // Update language submenu title and items
    if let Some(item) = menu.get("language") {
        if let Some(sub) = item.as_submenu() {
            let _ = sub.set_text(t("menu.language", current));
            for (id, lang) in [("lang_system", "system"), ("lang_zh", "zh"), ("lang_en", "en")] {
                if let Some(item) = sub.get(id) {
                    if let Some(m) = item.as_menuitem() {
                        let _ = m.set_text(build_lang_text(lang, current));
                    }
                }
            }
        }
    }
}
