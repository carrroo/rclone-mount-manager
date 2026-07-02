//! Language state — atomic global, persistence, and event emission.
//!
//! A single `AtomicU8` encodes the current language setting:
//! - `LANG_CODE_SYSTEM` (0) — follow OS locale
//! - `LANG_CODE_ZH` (1) — Chinese
//! - `LANG_CODE_EN` (2) — English

use std::sync::atomic::{AtomicU8, Ordering};

use tauri::{AppHandle, Emitter};

/// Language encoding constants.
pub const LANG_CODE_SYSTEM: u8 = 0;
pub const LANG_CODE_ZH: u8 = 1;
pub const LANG_CODE_EN: u8 = 2;

/// Global language state — stores one of the `LANG_CODE_*` constants.
static LANG: AtomicU8 = AtomicU8::new(LANG_CODE_SYSTEM);

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
    match LANG.load(Ordering::SeqCst) {
        LANG_CODE_ZH => "zh",
        LANG_CODE_EN => "en",
        _ => "system",
    }
}

/// Return the current language code for display purposes.
pub fn current_lang_code() -> u8 {
    LANG.load(Ordering::SeqCst)
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
    let lang = match LANG.load(Ordering::SeqCst) {
        LANG_CODE_ZH => "zh",
        LANG_CODE_EN => "en",
        _ if detect_system_is_zh() => "zh",
        _ => "en",
    };
    let _ = app.emit("language-changed", lang);
}

/// Apply a language change: update atomic state, menu text, persistence, and notify frontend.
pub fn apply_lang(app: &AppHandle, lang: &str) {
    let code = match lang {
        "zh" => LANG_CODE_ZH,
        "en" => LANG_CODE_EN,
        _ => LANG_CODE_SYSTEM,
    };
    LANG.store(code, Ordering::SeqCst);

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

    let code = match saved.trim() {
        "zh" => LANG_CODE_ZH,
        "en" => LANG_CODE_EN,
        _ => LANG_CODE_SYSTEM,
    };
    LANG.store(code, Ordering::SeqCst);
}
