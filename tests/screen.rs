#![cfg(not(target_arch = "wasm32"))]

mod common;

use dioxus::prelude::*;
use rust_app_ui::components::{EmptyNote, ErrorNote, Loading, Screen};
use rust_app_ui::error::ErrorKind;
use rust_app_ui::i18n::Language;

#[test]
fn screen_shows_loading_error_and_empty() {
    common::with_locale(Language::En, || {
        let loading = dioxus_ssr::render_element(rsx! { Loading {} });
        assert!(loading.contains("Loading"), "html was {loading}");

        let error = dioxus_ssr::render_element(rsx! { ErrorNote { kind: ErrorKind::Network } });
        assert!(error.to_lowercase().contains("network"), "html was {error}");

        let empty = dioxus_ssr::render_element(rsx! { EmptyNote { "nothing here" } });
        assert!(empty.contains("nothing here"), "html was {empty}");
    });
}

#[test]
fn screen_renders_title_subtitle_and_actions() {
    let html = dioxus_ssr::render_element(rsx! {
        Screen {
            title: "Tasks",
            subtitle: "Pending work",
            actions: rsx! { button { "Refresh" } },
            p { "content" }
        }
    });

    assert!(html.contains("Tasks"), "html was {html}");
    assert!(html.contains("Pending work"), "html was {html}");
    assert!(html.contains("Refresh"), "html was {html}");
    assert!(html.contains("content"), "html was {html}");
}
