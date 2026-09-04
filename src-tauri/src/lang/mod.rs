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
    apply_lang, current_lang, init_language_state,
};
pub use translate::{build_lang_text, t};
