//! Language module — manages app language state, native menu text, and i18n.
//!
//! Architecture:
//! - `state`    — AtomicU8 global for current language, persistence, and event emission
//! - `translate` — Translation helpers for native menu items
//! - `menu`     — Updates native menu text when language changes

mod menu;
mod state;
mod translate;

pub use menu::update_lang_menu_text;
pub use state::{
    apply_lang, current_lang, current_lang_code, init_language_state,
    LANG_CODE_SYSTEM, LANG_CODE_ZH, LANG_CODE_EN,
};
pub use translate::{build_lang_text, t};
