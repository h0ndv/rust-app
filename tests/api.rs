#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::rc::Rc;

use common::FakeTransport;
use contracts::{NewsKind, ResourceFilter, SettingsDto, Source, TaskFilter};
use rust_app_ui::api::Api;
use rust_app_ui::error::ErrorKind;
use rust_app_ui::transport::{Transport, UnsupportedTransport};
use serde_json::{Value, json};

fn api(fake: &FakeTransport) -> Api {
    Api::new(Rc::new(fake.clone()))
}

#[tokio::test]
async fn list_tasks_sends_the_lowercase_filter_and_reads_tasks() {
    let fake = FakeTransport::ok(json!([{
        "id": "activity-1",
        "course_id": "moodle-101",
        "course_name": "Data Mining",
        "title": "Lab 1",
        "due_at": 1_763_000_000_i64,
        "status": "overdue",
        "submitted": false,
        "grade": null,
        "url": "https://example.test/activity-1",
        "is_new": true
    }]));
    let api = api(&fake);

    let tasks = api.list_tasks(TaskFilter::Overdue).await.unwrap();

    assert_eq!(fake.last_command(), "list_tasks");
    assert_eq!(fake.last_args(), json!({ "filter": "overdue" }));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Lab 1");
    assert_eq!(tasks[0].status.as_deref(), Some("overdue"));
    assert!(tasks[0].is_new);
}

#[tokio::test]
async fn unauthorized_rejection_maps_to_the_ui_error() {
    let fake = FakeTransport::err(json!({
        "kind": "unauthorized",
        "message": "Session expired"
    }));
    let api = api(&fake);

    let error = api.list_grades(None).await.unwrap_err();

    assert_eq!(error.kind, ErrorKind::Unauthorized);
    assert_eq!(error.message, "Session expired");
    assert_eq!(error.key(), "errors.unauthorized");
    assert_eq!(error.to_string(), "Session expired");
}

#[tokio::test]
async fn an_unknown_error_kind_maps_to_internal() {
    let fake = FakeTransport::err(json!({
        "kind": "teapot",
        "message": "Short and stout"
    }));
    let api = api(&fake);

    let error = api.list_grades(None).await.unwrap_err();

    assert_eq!(error.kind, ErrorKind::Internal);
    assert_eq!(error.message, "Short and stout");
    assert_eq!(error.key(), "errors.internal");
}

#[tokio::test]
async fn get_course_sends_the_group_id() {
    let fake = FakeTransport::ok(json!({
        "group": {
            "id": "group-1",
            "code": "MAT100",
            "name": "Data Mining",
            "sections": []
        },
        "tasks": [],
        "resources": [],
        "grades": [],
        "schedule": []
    }));
    let api = api(&fake);

    let detail = api.get_course("group-1").await.unwrap();

    assert_eq!(fake.last_command(), "get_course");
    assert_eq!(fake.last_args(), json!({ "groupId": "group-1" }));
    assert_eq!(detail.group.id, "group-1");
}

#[tokio::test]
async fn download_file_sends_the_file_id() {
    let fake = FakeTransport::ok(Value::Null);
    let api = api(&fake);

    api.download_file("file-1").await.unwrap();

    assert_eq!(fake.last_command(), "download_file");
    assert_eq!(fake.last_args(), json!({ "fileId": "file-1" }));
}

#[tokio::test]
async fn optional_arguments_are_sent_as_null() {
    let fake = FakeTransport::ok(Value::Null);
    let api = api(&fake);

    let _ = api.connect(None).await;
    let _ = api.list_grades(None).await;

    assert_eq!(
        fake.calls(),
        vec![
            ("connect".to_string(), json!({ "source": null })),
            ("list_grades".to_string(), json!({ "term": null })),
        ]
    );
}

#[tokio::test]
async fn every_command_is_sent_with_its_expected_name_and_arguments() {
    let fake = FakeTransport::ok(Value::Null);
    let api = api(&fake);

    let _ = api.connect(Some(Source::Moodle)).await;
    let _ = api.session_state().await;
    let _ = api.refresh_session().await;
    let _ = api.start_sync().await;
    let _ = api.cancel_sync().await;
    let _ = api.sync_status().await;
    let _ = api.get_dashboard().await;
    let _ = api.has_local_data().await;
    let _ = api.list_courses().await;
    let _ = api.get_course("group-1").await;
    let _ = api.list_tasks(TaskFilter::NoData).await;
    let _ = api.list_resources(ResourceFilter::Downloaded).await;
    let _ = api.list_grades(Some("202620")).await;
    let _ = api.get_schedule().await;
    let _ = api.list_news().await;
    let _ = api.acknowledge(NewsKind::File, "file-1").await;
    let _ = api.search("mining").await;
    let _ = api.get_settings().await;
    let _ = api.recovery_notice().await;
    let _ = api.save_settings(SettingsDto::default()).await;
    let _ = api.set_language("es").await;
    let _ = api.open_in_app("https://example.test").await;
    let _ = api.download_file("file-1").await;
    let _ = api.clear_data().await;
    let _ = api.export_data().await;
    let _ = api.clear_downloads().await;
    let _ = api.set_window_theme("dark").await;
    let _ = api.notification_permission().await;
    let _ = api.request_notification_permission().await;
    let _ = api
        .send_notification("Lab due", "Lab 1 is due tomorrow")
        .await;

    let expected: Vec<(String, Value)> = vec![
        ("connect".into(), json!({ "source": "moodle" })),
        ("session_state".into(), json!({})),
        ("refresh_session".into(), json!({})),
        ("start_sync".into(), json!({})),
        ("cancel_sync".into(), json!({})),
        ("sync_status".into(), json!({})),
        ("get_dashboard".into(), json!({})),
        ("has_local_data".into(), json!({})),
        ("list_courses".into(), json!({})),
        ("get_course".into(), json!({ "groupId": "group-1" })),
        ("list_tasks".into(), json!({ "filter": "no_data" })),
        ("list_resources".into(), json!({ "filter": "downloaded" })),
        ("list_grades".into(), json!({ "term": "202620" })),
        ("get_schedule".into(), json!({})),
        ("list_news".into(), json!({})),
        (
            "acknowledge".into(),
            json!({ "kind": "file", "id": "file-1" }),
        ),
        ("search".into(), json!({ "query": "mining" })),
        ("get_settings".into(), json!({})),
        ("recovery_notice".into(), json!({})),
        (
            "save_settings".into(),
            json!({
                "settings": {
                    "sync_interval_minutes": 30,
                    "downloads_dir": "",
                    "theme": "system",
                    "notifications": true,
                    "language": "en"
                }
            }),
        ),
        ("set_language".into(), json!({ "language": "es" })),
        (
            "open_in_app".into(),
            json!({ "url": "https://example.test" }),
        ),
        ("download_file".into(), json!({ "fileId": "file-1" })),
        ("clear_data".into(), json!({})),
        ("export_data".into(), json!({})),
        ("clear_downloads".into(), json!({})),
        ("set_window_theme".into(), json!({ "theme": "dark" })),
        ("notification_permission".into(), json!({})),
        ("request_notification_permission".into(), json!({})),
        (
            "send_notification".into(),
            json!({ "title": "Lab due", "body": "Lab 1 is due tomorrow" }),
        ),
    ];

    assert_eq!(fake.calls(), expected);
}

#[test]
fn every_error_kind_has_a_localization_key() {
    let kinds = [
        (ErrorKind::Network, "errors.network"),
        (ErrorKind::Unauthorized, "errors.unauthorized"),
        (ErrorKind::Forbidden, "errors.forbidden"),
        (ErrorKind::Parse, "errors.parse"),
        (ErrorKind::NotFound, "errors.not_found"),
        (ErrorKind::RateLimited, "errors.rate_limited"),
        (ErrorKind::Internal, "errors.internal"),
    ];

    for (kind, key) in kinds {
        assert_eq!(rust_app_ui::error::ERROR_KEY[kind as usize], key);
    }
}

#[tokio::test]
async fn native_transport_is_unsupported() {
    let error = UnsupportedTransport
        .invoke("session_state", json!({}))
        .await
        .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Internal);
    assert_eq!(error.message, "no tauri transport");
}
