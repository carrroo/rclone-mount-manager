//! Translation helpers for native macOS menu items.
//!
//! These are separate from the frontend's vue-i18n system because
//! native menus are rendered by the OS and need Rust-side strings.

use super::state::resolved_lang_from;

/// Return a checkmark prefix for the currently active language option.
fn lang_prefix(is_active: bool) -> &'static str {
    if is_active { "✓ " } else { "  " }
}

/// Build the display text for a language menu item.
pub fn build_lang_text(lang: &str, current: &str) -> String {
    let rl = resolved_lang_from(current);
    match lang {
        "system" => format!("{}{}", lang_prefix(current == "system"), if rl == "zh" { "跟随系统" } else { "Follow System" }),
        "zh" => format!("{}中文", lang_prefix(current == "zh")),
        "en" => format!("{}English", lang_prefix(current == "en")),
        _ => lang.to_string(),
    }
}

/// Translate a key for the native menu using the current language.
pub fn t(key: &str, lang: &str) -> String {
    let rl = resolved_lang_from(lang);
    match key {
        "menu.app_title" => "Rclone Mount Manager".to_string(),
        "menu.language" => if rl == "zh" { "语言".to_string() } else { "Language".to_string() },
        "menu.open" => if rl == "zh" { "打开".to_string() } else { "Open".to_string() },
        "menu.quit" => if rl == "zh" { "退出".to_string() } else { "Quit".to_string() },
        "menu.edit" => if rl == "zh" { "编辑".to_string() } else { "Edit".to_string() },
        _ => key.to_string(),
    }
}
