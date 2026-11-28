#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::rc::Rc;

use common::{FakeToasts, FakeTransport};
use rust_app_ui::api::Api;
use rust_app_ui::components::ToastKind;
use rust_app_ui::i18n::t;
use rust_app_ui::open::open_in_app;
use serde_json::json;

#[tokio::test]
async fn open_failure_shows_a_localized_toast() {
    let api = Api::new(Rc::new(FakeTransport::err(json!({
        "kind": "forbidden", "message": "Link not allowed"
    }))));
    let toasts = FakeToasts::default();

    open_in_app(&api, &toasts, "https://example.com").await;

    let items = toasts.items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, t("errors.forbidden"));
    assert_eq!(items[0].kind, ToastKind::Error);
    assert_eq!(items[0].description, None);
}

#[tokio::test]
async fn open_success_pushes_nothing() {
    let api = Api::new(Rc::new(FakeTransport::ok(serde_json::Value::Null)));
    let toasts = FakeToasts::default();

    open_in_app(&api, &toasts, "https://example.com").await;

    assert!(toasts.items().is_empty());
}

#[tokio::test]
async fn a_missing_resource_maps_to_the_not_found_copy() {
    let api = Api::new(Rc::new(FakeTransport::err(json!({
        "kind": "not_found", "message": "__missing__"
    }))));
    let toasts = FakeToasts::default();

    open_in_app(&api, &toasts, "https://example.com").await;

    assert_eq!(toasts.items()[0].title, t("errors.not_found"));
}
