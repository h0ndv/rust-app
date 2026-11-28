#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::rc::Rc;

use common::FakeTransport;
use contracts::NewsItemDto;
use dioxus::prelude::*;
use rust_app_ui::api::Api;
use rust_app_ui::features::news::{NewsItemBody, NewsView, mark_read, news_link};
use rust_app_ui::i18n::Language;
use serde_json::{Value, json};

fn announcement() -> NewsItemDto {
    NewsItemDto {
        id: "moodle:1:2".into(),
        course: Some("Course".into()),
        group_id: Some("g1".into()),
        title: "Announcement".into(),
        body: "Body".into(),
        published_at: None,
        kind: "announcement".into(),
    }
}

fn file() -> NewsItemDto {
    NewsItemDto {
        id: "file-1".into(),
        title: "Guia".into(),
        kind: "file".into(),
        ..announcement()
    }
}

fn activity() -> NewsItemDto {
    NewsItemDto {
        id: "a1".into(),
        title: "Lab 1".into(),
        kind: "activity".into(),
        ..announcement()
    }
}

#[component]
fn BodyHarness(item: NewsItemDto, #[props(default)] busy: bool) -> Element {
    rsx! {
        NewsItemBody { item, busy, on_acknowledge: |_| {} }
    }
}

fn render_body(item: NewsItemDto, busy: bool) -> String {
    dioxus_ssr::render_element(rsx! {
        BodyHarness { item, busy }
    })
}

#[component]
fn ViewHarness(items: Vec<NewsItemDto>, #[props(default)] busy: Option<String>) -> Element {
    rsx! {
        NewsView { items, busy, on_acknowledge: |_| {} }
    }
}

fn render_view(items: Vec<NewsItemDto>, busy: Option<&str>) -> String {
    let busy = busy.map(str::to_string);

    dioxus_ssr::render_element(rsx! {
        ViewHarness { items, busy }
    })
}

fn loose(mut item: NewsItemDto) -> NewsItemDto {
    item.group_id = None;
    item
}

#[test]
fn announcement_links_without_focus_and_file_links_with_focus() {
    assert_eq!(news_link(&announcement()), "/ramos/g1");
    assert_eq!(news_link(&file()), "/ramos/g1?focus=file-1");
}

#[test]
fn activities_link_with_focus() {
    assert_eq!(news_link(&activity()), "/ramos/g1?focus=a1");
}

#[test]
fn an_item_without_a_course_links_to_the_course_list() {
    assert_eq!(news_link(&loose(announcement())), "/ramos");
}

#[test]
fn the_item_shows_the_kind_badge_title_course_body_and_action() {
    let html = common::with_locale(Language::En, || render_body(activity(), false));

    assert!(html.contains(">Activity<"), "html was {html}");
    assert!(html.contains("Lab 1"), "html was {html}");
    assert!(html.contains("Course"), "html was {html}");
    assert!(html.contains("Body"), "html was {html}");
    assert!(html.contains("Mark as read"), "html was {html}");
    assert!(!html.contains("disabled=true"), "html was {html}");
}

#[test]
fn the_course_and_the_body_only_show_when_present() {
    let mut bare = activity();
    bare.course = None;
    bare.body = String::new();

    let html = common::with_locale(Language::En, || render_body(bare, false));

    assert!(html.contains("Lab 1"), "html was {html}");
    assert!(!html.contains("Course"), "html was {html}");
    assert!(!html.contains("Body"), "html was {html}");

    let mut empty = activity();
    empty.course = Some(String::new());

    let html = common::with_locale(Language::En, || render_body(empty, false));

    assert!(
        !html.contains("text-xs text-muted-foreground"),
        "html was {html}"
    );
}

#[test]
fn an_unknown_kind_falls_back_to_other() {
    let mut strange = activity();
    strange.kind = "mystery".into();

    let html = common::with_locale(Language::En, || render_body(strange, false));

    assert!(html.contains(">Other<"), "html was {html}");
}

#[test]
fn the_mark_read_button_disables_while_its_item_is_busy() {
    let busy = common::with_locale(Language::En, || render_body(announcement(), true));
    let idle = common::with_locale(Language::En, || render_body(announcement(), false));

    assert!(busy.contains("disabled=true"), "html was {busy}");
    assert!(!idle.contains("disabled=true"), "html was {idle}");
}

#[test]
fn the_list_keeps_the_item_order() {
    let html = common::with_locale(Language::En, || {
        render_view(vec![loose(file()), loose(activity())], None)
    });

    let first = html.find("Guia").expect("the first title");
    let second = html.find("Lab 1").expect("the second title");
    let buttons = html.matches("Mark as read").count();

    assert!(first < second, "html was {html}");
    assert_eq!(buttons, 2, "html was {html}");
}

#[test]
fn only_the_busy_item_is_disabled() {
    let html = common::with_locale(Language::En, || {
        render_view(vec![loose(announcement()), loose(file())], Some("file-1"))
    });

    assert_eq!(html.matches("disabled=true").count(), 1, "html was {html}");
}

#[test]
fn an_empty_list_shows_the_empty_note() {
    let html = common::with_locale(Language::En, || render_view(Vec::new(), None));

    assert!(
        html.contains("No news. No course published new announcements or left resources unopened."),
        "html was {html}"
    );
}

#[test]
fn the_copy_follows_the_locale() {
    let html = common::with_locale(Language::Es, || render_body(file(), false));

    assert!(html.contains(">Archivo<"), "html was {html}");
    assert!(html.contains("Marcar como leído"), "html was {html}");
}

#[tokio::test]
async fn marking_an_item_as_read_sends_its_kind_and_id() {
    let fake = FakeTransport::ok(Value::Null);
    let api = Api::new(Rc::new(fake.clone()));

    assert!(mark_read(&api, &announcement()).await);
    assert_eq!(fake.last_command(), "acknowledge");
    assert_eq!(
        fake.last_args(),
        json!({ "kind": "announcement", "id": "moodle:1:2" })
    );

    assert!(mark_read(&api, &file()).await);
    assert_eq!(fake.last_args(), json!({ "kind": "file", "id": "file-1" }));

    assert!(mark_read(&api, &activity()).await);
    assert_eq!(fake.last_args(), json!({ "kind": "activity", "id": "a1" }));
}

#[tokio::test]
async fn a_failed_acknowledgement_reports_it_did_not_land() {
    let fake = FakeTransport::err(json!({
        "kind": "network", "message": "offline"
    }));
    let api = Api::new(Rc::new(fake));

    assert!(!mark_read(&api, &announcement()).await);
}
