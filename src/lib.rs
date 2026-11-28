pub mod api;
pub mod app;
pub mod components;
pub mod domain;
pub mod error;
pub mod events;
pub mod features;
pub mod format;
pub mod i18n;
pub mod icons;
pub mod notifications;
pub mod open;
pub mod router;
pub mod state;
pub mod theme;
pub mod transport;

pub use app::App;

rust_i18n::i18n!("locales", fallback = "en");

#[cfg(target_arch = "wasm32")]
pub fn launch() {
    dioxus::launch(App);
}
