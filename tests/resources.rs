#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::rc::Rc;

use common::{FakeToasts, FakeTransport};
use contracts::ResourceDto;
use dioxus::prelude::*;
use rust_app_ui::api::Api;
use rust_app_ui::components::ToastKind;
use rust_app_ui::features::resources::{ResourcesView, download_file};
use rust_app_ui::i18n::{Language, t};
use serde_json::{Value, json};

fn file(id: &str, course: Option<&str>) -> ResourceDto {
    ResourceDto {
        id: id.into(),
        course: course.map(str::to_string),
        title: id.into(),
        kind: "file".into(),
        size: Some(2048),
        url: format!("https://example.test/{id}"),
        ..Default::default()
    }
}

#[component]
fn ResourcesHarness(
    resources: Vec<ResourceDto>,
    #[props(default)] busy: Option<String>,
) -> Element {
    rsx! {
        ResourcesView {
            resources,
            busy,
            on_open: |_| {},
            on_download: |_| {},
        }
    }
}

fn render(resources: Vec<ResourceDto>) -> String {
    dioxus_ssr::render_element(rsx! {
        ResourcesHarness { resources, busy: None }
    })
}

fn render_busy(resources: Vec<ResourceDto>, busy: Option<&str>) -> String {
    let busy = busy.map(str::to_string);

    dioxus_ssr::render_element(rsx! {
        ResourcesHarness { resources, busy }
    })
}

#[test]
fn groups_by_course_and_shows_download_state() {
    let mut saved = file("saved", Some("Curso A"));
    saved.downloaded = true;

    let html = common::with_locale(Language::En, || {
        render(vec![file("fresh", Some("Curso A")), saved])
    });

    assert!(html.contains("Curso A"), "html was {html}");
    assert!(html.contains("Downloaded"), "html was {html}");
    assert!(html.contains("Download"), "html was {html}");
}

#[test]
fn groups_keep_the_first_seen_order() {
    let html = common::with_locale(Language::En, || {
        render(vec![
            file("b1", Some("Course B")),
            file("a1", Some("Course A")),
            file("b2", Some("Course B")),
        ])
    });

    let positions = ["Course B", "b1", "b2", "Course A", "a1"].map(|needle| {
        html.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing in {html}"))
    });
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "order was {positions:?} in {html}"
    );
    assert_eq!(html.matches("Course B").count(), 1, "html was {html}");
}

#[test]
fn a_missing_course_falls_back_to_the_no_course_group() {
    let html = common::with_locale(Language::En, || {
        render(vec![file("loose", None), file("filed", Some("Course A"))])
    });

    assert!(html.contains("No course"), "html was {html}");
    assert!(html.contains("Course A"), "html was {html}");
}

#[test]
fn rows_show_the_title_new_badge_kind_and_size() {
    let mut fresh = file("slides", Some("Course A"));
    fresh.is_new = true;
    let mut strange = file("misc", Some("Course A"));
    strange.kind = "mystery".into();
    strange.size = None;

    let html = common::with_locale(Language::En, || render(vec![fresh, strange]));

    assert!(html.contains("slides"), "html was {html}");
    assert_eq!(html.matches(">New<").count(), 1, "html was {html}");
    assert!(html.contains(">File<"), "html was {html}");
    assert!(html.contains(">2 KB<"), "html was {html}");
    assert!(html.contains(">Other<"), "html was {html}");
}

#[test]
fn only_pending_files_offer_a_download_button() {
    let mut saved = file("saved", Some("Course A"));
    saved.downloaded = true;
    let pending = file("pending", Some("Course A"));

    let html = common::with_locale(Language::En, || render(vec![saved, pending]));

    assert_eq!(html.matches(">Downloaded<").count(), 1, "html was {html}");
    assert_eq!(html.matches(">Download<").count(), 1, "html was {html}");
}

#[test]
fn every_row_offers_open() {
    let html = common::with_locale(Language::En, || {
        render(vec![
            file("a", Some("Course A")),
            file("b", Some("Course A")),
        ])
    });

    assert_eq!(html.matches(">Open<").count(), 2, "html was {html}");
}

#[test]
fn the_busy_row_disables_download_and_shows_progress_copy() {
    let html = common::with_locale(Language::En, || {
        render_busy(
            vec![
                file("busy", Some("Course A")),
                file("idle", Some("Course A")),
            ],
            Some("busy"),
        )
    });

    assert_eq!(html.matches("disabled=true").count(), 1, "html was {html}");
    assert!(html.contains("Downloading…"), "html was {html}");
    assert_eq!(html.matches(">Download<").count(), 1, "html was {html}");
}

#[test]
fn an_empty_list_shows_the_empty_note() {
    let html = common::with_locale(Language::En, || render(Vec::new()));

    assert!(html.contains("No resources synced yet."), "html was {html}");
}

#[test]
fn the_copy_follows_the_locale() {
    let mut saved = file("saved", None);
    saved.is_new = true;
    saved.kind = "archivo".into();
    saved.downloaded = true;
    let pending = file("pending", Some("Ramo A"));

    let html = common::with_locale(Language::Es, || render(vec![saved, pending]));

    assert!(html.contains("Sin ramo"), "html was {html}");
    assert!(html.contains("Nuevo"), "html was {html}");
    assert!(html.contains("Archivo"), "html was {html}");
    assert!(html.contains("Descargado"), "html was {html}");
    assert!(html.contains("Descargar"), "html was {html}");
    assert!(html.contains("Abrir"), "html was {html}");
}

#[tokio::test]
async fn a_successful_download_sends_the_file_id() {
    let fake = FakeTransport::ok(Value::Null);
    let api = Api::new(Rc::new(fake.clone()));
    let toasts = FakeToasts::default();

    let completed = download_file(&api, &toasts, "file-7").await;

    assert!(completed);
    assert_eq!(fake.last_command(), "download_file");
    assert_eq!(fake.last_args(), json!({ "fileId": "file-7" }));
    assert!(toasts.items().is_empty());
}

#[tokio::test]
async fn a_failed_download_shows_a_localized_toast() {
    let fake = FakeTransport::err(json!({
        "kind": "network", "message": "offline"
    }));
    let api = Api::new(Rc::new(fake));
    let toasts = FakeToasts::default();

    let completed = download_file(&api, &toasts, "file-7").await;

    assert!(!completed);
    let items = toasts.items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, t("errors.network"));
    assert_eq!(items[0].kind, ToastKind::Error);
}
