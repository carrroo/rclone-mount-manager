use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

pub static LANG_SYSTEM: AtomicBool = AtomicBool::new(true);
pub static LANG_ZH: AtomicBool = AtomicBool::new(false);
pub static LANG_EN: AtomicBool = AtomicBool::new(false);

pub fn detect_system_is_zh() -> bool {
    std::env::var("LANG")
        .map(|l| l.starts_with("zh"))
        .unwrap_or(false)
}

pub fn current_lang() -> &'static str {
    if LANG_ZH.load(Ordering::SeqCst) {
        "zh"
    } else if LANG_EN.load(Ordering::SeqCst) {
        "en"
    } else {
        "system"
    }
}

pub fn resolved_lang() -> &'static str {
    match current_lang() {
        "system" => if detect_system_is_zh() { "zh" } else { "en" },
        other => other,
    }
}

fn resolved_lang_from(current: &str) -> &'static str {
    match current {
        "system" => if detect_system_is_zh() { "zh" } else { "en" },
        "zh" => "zh",
        "en" => "en",
        _ => "en",
    }
}

fn lang_prefix(is_active: bool) -> &'static str {
    if is_active { "✓ " } else { "  " }
}

pub fn build_lang_text(lang: &str, current: &str) -> String {
    let rl = resolved_lang_from(current);
    match lang {
        "system" => format!("{}{}", lang_prefix(current == "system"), if rl == "zh" { "跟随系统" } else { "Follow System" }),
        "zh" => format!("{}中文", lang_prefix(current == "zh")),
        "en" => format!("{}English", lang_prefix(current == "en")),
        _ => lang.to_string(),
    }
}

pub fn t(key: &str, lang: &str) -> String {
    let rl = resolved_lang_from(lang);
    match key {
        "menu.app_title" => "Rclone Mount Manager".to_string(),
        "menu.language" => if rl == "zh" { "语言".to_string() } else { "Language".to_string() },
        "menu.open" => if rl == "zh" { "打开".to_string() } else { "Open".to_string() },
        "menu.quit" => if rl == "zh" { "退出".to_string() } else { "Quit".to_string() },
        _ => key.to_string(),
    }
}

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

pub fn save_language_setting(lang: &str) {
    if let Some(home) = dirs::home_dir() {
        let dir = home.join(".config/rclone-mount-manager");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("language"), lang);
    }
}

pub fn emit_language_change(app: &AppHandle) {
    let lang = if LANG_ZH.load(Ordering::SeqCst) {
        "zh".to_string()
    } else if LANG_EN.load(Ordering::SeqCst) {
        "en".to_string()
    } else if detect_system_is_zh() {
        "zh".to_string()
    } else {
        "en".to_string()
    };
    let _ = app.emit("language-changed", lang);
}

pub fn apply_lang(app: &AppHandle, lang: &str) {
    let (sys, zh, en) = match lang {
        "zh" => (false, true, false),
        "en" => (false, false, true),
        _ => (true, false, false),
    };
    LANG_SYSTEM.store(sys, Ordering::SeqCst);
    LANG_ZH.store(zh, Ordering::SeqCst);
    LANG_EN.store(en, Ordering::SeqCst);

    update_lang_menu_text(app, lang);
    save_language_setting(lang);
    emit_language_change(app);
}

pub fn init_language_state() {
    let saved = std::fs::read_to_string(
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join(".config/rclone-mount-manager/language"),
    )
    .unwrap_or_default();

    // Reset all flags first to ensure mutual exclusion
    LANG_SYSTEM.store(false, Ordering::SeqCst);
    LANG_ZH.store(false, Ordering::SeqCst);
    LANG_EN.store(false, Ordering::SeqCst);

    match saved.trim() {
        "zh" => LANG_ZH.store(true, Ordering::SeqCst),
        "en" => LANG_EN.store(true, Ordering::SeqCst),
        _ => LANG_SYSTEM.store(true, Ordering::SeqCst),
    }
}
