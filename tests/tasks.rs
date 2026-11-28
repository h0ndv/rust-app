#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::TaskDto;
use dioxus::prelude::*;
use rust_app_ui::features::tasks::TasksView;
use rust_app_ui::i18n::Language;

const DUE_AT: i64 = 1_795_521_600;

fn task(id: &str, status: &str) -> TaskDto {
    TaskDto {
        id: id.into(),
        course_id: "moodle:1".into(),
        course_name: "Minería de Datos".into(),
        title: id.into(),
        due_at: Some(DUE_AT),
        status: Some(status.into()),
        ..Default::default()
    }
}

#[component]
fn TasksHarness(tasks: Vec<TaskDto>) -> Element {
    rsx! {
        TasksView { tasks, on_open: |_| {} }
    }
}

fn render(tasks: Vec<TaskDto>) -> String {
    dioxus_ssr::render_element(rsx! {
        TasksHarness { tasks }
    })
}

#[test]
fn groups_render_only_when_they_have_items() {
    let html = common::with_locale(Language::En, || {
        render(vec![task("a", "overdue"), task("b", "graded")])
    });

    assert!(html.contains("Overdue"), "html was {html}");
    assert!(html.contains("Graded"), "html was {html}");
    assert!(!html.contains("Submitted"), "html was {html}");
}

#[test]
fn groups_keep_the_react_order() {
    let html = common::with_locale(Language::En, || {
        render(vec![
            task("a", "graded"),
            task("b", "informational"),
            task("c", "pending"),
            task("d", "overdue"),
            task("e", "unconfirmed"),
            task("f", "submitted"),
            task("g", "no_data"),
        ])
    });

    let headings = [
        "Overdue",
        "Pending",
        "Submitted",
        "Graded",
        "No date",
        "To confirm",
        "Informational",
    ];
    let positions: Vec<usize> = headings
        .iter()
        .map(|heading| {
            html.find(heading)
                .unwrap_or_else(|| panic!("{heading} missing in {html}"))
        })
        .collect();

    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "order was {positions:?} in {html}"
    );
}

#[test]
fn rows_show_the_title_course_date_and_new_badge() {
    let mut fresh = task("Taller", "pending");
    fresh.is_new = true;
    let old = task("Lectura", "pending");

    let html = common::with_locale(Language::En, || render(vec![fresh, old]));

    assert!(html.contains("Taller"), "html was {html}");
    assert!(html.contains("Lectura"), "html was {html}");
    assert_eq!(
        html.matches("Minería de Datos").count(),
        2,
        "html was {html}"
    );
    assert!(html.contains("Nov 24, 12:00 PM"), "html was {html}");
    assert_eq!(html.matches(">New<").count(), 1, "html was {html}");
}

#[test]
fn a_grade_shows_only_when_present() {
    let mut graded = task("Control 1", "graded");
    graded.grade = Some(6.5);
    let pending = task("Control 2", "pending");

    let html = common::with_locale(Language::En, || render(vec![graded, pending]));

    assert_eq!(html.matches(">6.5<").count(), 1, "html was {html}");
    assert!(!html.contains("Not published"), "html was {html}");
}

#[test]
fn the_open_action_needs_a_url() {
    let mut linked = task("Con enlace", "pending");
    linked.url = Some("https://example.test/task".into());
    let plain = task("Sin enlace", "pending");

    let html = common::with_locale(Language::En, || render(vec![linked, plain]));

    assert_eq!(html.matches(">Open<").count(), 1, "html was {html}");
}

#[test]
fn an_empty_list_shows_the_empty_note() {
    let html = common::with_locale(Language::En, || render(Vec::new()));

    assert!(
        html.contains("No tasks match this filter."),
        "html was {html}"
    );
}

#[test]
fn statuses_outside_the_main_groups_render() {
    let html = common::with_locale(Language::En, || {
        render(vec![task("Course guide", "informational")])
    });

    assert!(html.contains("Informational"), "html was {html}");
    assert!(html.contains("Course guide"), "html was {html}");
}

#[test]
fn the_copy_and_the_grade_follow_the_locale() {
    let mut graded = task("Control 1", "atrasada");
    graded.grade = Some(6.5);
    graded.is_new = true;
    graded.url = Some("https://example.test/task".into());

    let html = common::with_locale(Language::Es, || render(vec![graded]));

    assert!(html.contains("Atrasada"), "html was {html}");
    assert!(html.contains("Nuevo"), "html was {html}");
    assert!(html.contains("6,5"), "html was {html}");
    assert!(html.contains("Abrir"), "html was {html}");
    assert!(!html.contains("6.5"), "html was {html}");
}
