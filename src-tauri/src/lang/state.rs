//! Language state — atomic globals, persistence, and event emission.
//!
//! Three mutually exclusive AtomicBool flags track the current language:
//! `LANG_SYSTEM` (follow OS), `LANG_ZH`, `LANG_EN`.
//! Exactly one is `true` at any time.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter};

/// Global language flags — exactly one is true at a time.
pub static LANG_SYSTEM: AtomicBool = AtomicBool::new(true);
pub static LANG_ZH: AtomicBool = AtomicBool::new(false);
pub static LANG_EN: AtomicBool = AtomicBool::new(false);

/// Detect whether the system locale is Chinese.
///
/// Uses `sys-locale` which reads NSLocale on macOS — works in GUI apps
/// where the `LANG` environment variable is typically unset.
pub fn detect_system_is_zh() -> bool {
    sys_locale::get_locale()
        .map(|l| l.starts_with("zh"))
        .unwrap_or(false)
}

/// Return the raw language setting ("zh", "en", or "system").
pub fn current_lang() -> &'static str {
    if LANG_ZH.load(Ordering::SeqCst) {
        "zh"
    } else if LANG_EN.load(Ordering::SeqCst) {
        "en"
    } else {
        "system"
    }
}

/// Resolve the effective locale ("zh" or "en") by expanding "system".
#[allow(dead_code)]
pub fn resolved_lang() -> &'static str {
    resolved_lang_from(current_lang())
}

/// Internal: resolve a language code to "zh" or "en".
pub(crate) fn resolved_lang_from(current: &str) -> &'static str {
    match current {
        "system" => if detect_system_is_zh() { "zh" } else { "en" },
        "zh" => "zh",
        "en" => "en",
        _ => "en",
    }
}

/// Persist the language setting to `~/.config/rclone-mount-manager/language`.
fn save_language_setting(lang: &str) {
    if let Some(home) = dirs::home_dir() {
        let dir = home.join(".config/rclone-mount-manager");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("language"), lang);
    }
}

/// Emit a `language-changed` event to the frontend with the resolved locale.
fn emit_language_change(app: &AppHandle) {
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

/// Apply a language change: update atomic state, menu text, persistence, and notify frontend.
pub fn apply_lang(app: &AppHandle, lang: &str) {
    let (sys, zh, en) = match lang {
        "zh" => (false, true, false),
        "en" => (false, false, true),
        _ => (true, false, false),
    };
    LANG_SYSTEM.store(sys, Ordering::SeqCst);
    LANG_ZH.store(zh, Ordering::SeqCst);
    LANG_EN.store(en, Ordering::SeqCst);

    super::menu::update_lang_menu_text(app, lang);
    save_language_setting(lang);
    emit_language_change(app);
}

/// Initialize language state from the persisted setting file.
/// Called once at app startup before the Tauri builder runs.
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
