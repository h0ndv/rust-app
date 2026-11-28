#![cfg(not(target_arch = "wasm32"))]

use rust_app_ui::theme::{ThemePreference, normalize_theme, resolve_theme};

#[test]
fn system_follows_the_media_query() {
    assert_eq!(resolve_theme("system", true), "dark");
    assert_eq!(resolve_theme("system", false), "light");
}

#[test]
fn explicit_preferences_ignore_the_system() {
    assert_eq!(resolve_theme("dark", false), "dark");
    assert_eq!(resolve_theme("light", true), "light");
}

#[test]
fn unknown_values_fall_back_to_system() {
    assert_eq!(resolve_theme("nonsense", true), "dark");
    assert_eq!(resolve_theme("", false), "light");
    assert_eq!(normalize_theme("nonsense"), ThemePreference::System);
    assert_eq!(normalize_theme("light"), ThemePreference::Light);
    assert_eq!(normalize_theme("dark"), ThemePreference::Dark);
}
