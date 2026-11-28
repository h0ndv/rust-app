#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::{DashboardDto, ResourceDto, ScheduleEventDto, TaskDto};
use dioxus::prelude::*;
use rust_app_ui::features::home::HomeSections;
use rust_app_ui::i18n::Language;

fn task(id: &str, title: &str, due_at: Option<i64>) -> TaskDto {
    TaskDto {
        id: id.into(),
        course_id: "course-1".into(),
        course_name: "Data Mining".into(),
        title: title.into(),
        due_at,
        ..Default::default()
    }
}

fn event(id: &str, title: &str, start: &str, end: &str, room: Option<&str>) -> ScheduleEventDto {
    ScheduleEventDto {
        id: id.into(),
        title: title.into(),
        start: Some(start.into()),
        end: Some(end.into()),
        room: room.map(str::to_string),
        ..Default::default()
    }
}

fn resource(id: &str, title: &str) -> ResourceDto {
    ResourceDto {
        id: id.into(),
        title: title.into(),
        kind: "file".into(),
        url: "https://example.test/file.pdf".into(),
        ..Default::default()
    }
}

fn render(dashboard: DashboardDto) -> String {
    dioxus_ssr::render_element(rsx! {
        HomeSections { dashboard }
    })
}

#[test]
fn an_empty_dashboard_shows_the_empty_note() {
    let html = common::with_locale(Language::En, || render(DashboardDto::default()));

    assert!(
        html.contains("No data yet. Sync to bring your courses and tasks."),
        "html was {html}"
    );
    for heading in ["Overdue", "Next 7 days", "Next classes", "New resources"] {
        assert!(!html.contains(heading), "{heading} in {html}");
    }
}

#[test]
fn sections_appear_only_when_they_have_rows() {
    common::with_locale(Language::En, || {
        let headings = ["Overdue", "Next 7 days", "Next classes", "New resources"];
        let cases = [
            (
                DashboardDto {
                    overdue: vec![task("a", "Lab 1", Some(1_795_521_600))],
                    ..Default::default()
                },
                0,
            ),
            (
                DashboardDto {
                    due_soon: vec![task("b", "Essay 2", Some(1_795_521_600))],
                    ..Default::default()
                },
                1,
            ),
            (
                DashboardDto {
                    next_classes: vec![event("c", "Calculo", "08:30", "09:50", None)],
                    ..Default::default()
                },
                2,
            ),
            (
                DashboardDto {
                    new_resources: vec![resource("d", "Week 3 slides")],
                    ..Default::default()
                },
                3,
            ),
        ];

        for (dashboard, shown) in cases {
            let html = render(dashboard);
            for (index, heading) in headings.iter().enumerate() {
                assert_eq!(
                    html.contains(heading),
                    index == shown,
                    "{heading} in {html}"
                );
            }
        }
    });
}

#[test]
fn task_rows_show_the_title_course_and_date() {
    let due_at = 1_795_521_600;
    let dashboard = DashboardDto {
        overdue: vec![task("a", "Lab 1", Some(due_at))],
        due_soon: vec![task("b", "Essay 2", Some(due_at))],
        ..Default::default()
    };

    let html = common::with_locale(Language::En, || render(dashboard));

    assert!(html.contains("Lab 1"), "html was {html}");
    assert!(html.contains("Essay 2"), "html was {html}");
    assert!(
        html.contains("Data Mining · Nov 24, 12:00 PM"),
        "html was {html}"
    );
    assert_eq!(
        html.matches("Data Mining · Nov 24, 12:00 PM").count(),
        2,
        "html was {html}"
    );
}

#[test]
fn class_rows_show_time_and_room_only_when_present() {
    let dashboard = DashboardDto {
        next_classes: vec![
            event("a", "Calculo", "08:30", "09:50", Some("B-12")),
            event("b", "Fisica", "10:00", "11:20", None),
        ],
        ..Default::default()
    };

    let html = common::with_locale(Language::En, || render(dashboard));

    assert!(html.contains("Calculo"), "html was {html}");
    assert!(html.contains("Fisica"), "html was {html}");
    assert!(html.contains("08:30 - 09:50"), "html was {html}");
    assert!(html.contains("10:00 - 11:20"), "html was {html}");
    assert!(html.contains("B-12"), "html was {html}");
    assert_eq!(html.matches("B-12").count(), 1, "html was {html}");
    assert!(!html.contains("None"), "html was {html}");
}

#[test]
fn resource_rows_show_the_title() {
    let dashboard = DashboardDto {
        new_resources: vec![
            resource("a", "Week 3 slides"),
            resource("b", "Reading guide"),
        ],
        ..Default::default()
    };

    let html = common::with_locale(Language::En, || render(dashboard));

    assert!(html.contains("Week 3 slides"), "html was {html}");
    assert!(html.contains("Reading guide"), "html was {html}");
    assert!(html.contains("New resources"), "html was {html}");
}

#[test]
fn the_empty_note_follows_the_locale() {
    let html = common::with_locale(Language::Es, || render(DashboardDto::default()));

    assert!(
        html.contains("Todavía no hay datos. Sincroniza para traer los ramos y las tareas."),
        "html was {html}"
    );
}
