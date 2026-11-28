#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::str::FromStr;

use contracts::{
    ActivityRow, CourseDetailDto, CourseGroupDto, CourseSectionDto, FileRow, GradeRow, ScheduleRow,
};
use dioxus::prelude::*;
use rust_app_ui::features::course_detail::CourseDetailView;
use rust_app_ui::i18n::Language;
use rust_app_ui::router::Route;

const DUE_AT: i64 = 1_795_521_600;

fn section(id: &str, section: Option<&str>, teacher: Option<&str>) -> CourseSectionDto {
    CourseSectionDto {
        id: id.into(),
        fullname: "Mineria de Datos".into(),
        section: section.map(str::to_string),
        teacher: teacher.map(str::to_string),
        source: "moodle".into(),
        ..Default::default()
    }
}

fn detail() -> CourseDetailDto {
    CourseDetailDto {
        group: CourseGroupDto {
            id: "g1".into(),
            code: "IGR-001".into(),
            name: "Mineria de Datos".into(),
            sections: vec![
                section("s1", Some("1"), Some("Ana Perez")),
                section("s2", None, None),
            ],
        },
        tasks: vec![
            ActivityRow {
                id: "a1".into(),
                name: "Laboratorio 1".into(),
                status: Some("pendiente".into()),
                due_at: Some(DUE_AT),
                ..Default::default()
            },
            ActivityRow {
                id: "a2".into(),
                name: "Lectura".into(),
                status: None,
                due_at: None,
                acknowledged_at: Some("2026-03-01T12:00:00Z".into()),
                ..Default::default()
            },
        ],
        resources: vec![FileRow {
            id: "f1".into(),
            name: "Guia.pdf".into(),
            url: "https://example.test/guia.pdf".into(),
            ..Default::default()
        }],
        grades: vec![GradeRow {
            id: "gr1".into(),
            name: "Control 1".into(),
            grade: Some(6.5),
            max_grade: Some(7.0),
            source: "banner".into(),
        }],
        schedule: vec![
            ScheduleRow {
                id: "h1".into(),
                title: "Catedra".into(),
                start_time: Some("08:30".into()),
                end_time: Some("09:50".into()),
                room: Some("B-12".into()),
                ..Default::default()
            },
            ScheduleRow {
                id: "h2".into(),
                title: String::new(),
                start_time: Some("10:00".into()),
                end_time: Some("11:20".into()),
                room: None,
                ..Default::default()
            },
        ],
    }
}

fn empty_detail() -> CourseDetailDto {
    CourseDetailDto {
        group: CourseGroupDto {
            id: "g1".into(),
            code: "IGR-001".into(),
            name: "Mineria de Datos".into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[component]
fn DetailHarness(detail: CourseDetailDto, #[props(default)] focus: Option<String>) -> Element {
    rsx! {
        CourseDetailView { detail, focus, on_open: |_| {} }
    }
}

fn render(detail: CourseDetailDto, focus: Option<&str>) -> String {
    let focus = focus.map(str::to_string);

    dioxus_ssr::render_element(rsx! {
        DetailHarness { detail, focus }
    })
}

#[test]
fn sections_show_their_label_and_teacher_or_the_fallbacks() {
    let html = common::with_locale(Language::En, || render(detail(), None));

    assert!(html.contains("Sections"), "html was {html}");
    assert!(html.contains("1 · Ana Perez"), "html was {html}");
    assert!(html.contains("No section · No teacher"), "html was {html}");
}

#[test]
fn tasks_show_the_name_new_badge_status_and_date() {
    let html = common::with_locale(Language::En, || render(detail(), None));

    assert!(html.contains("Tasks"), "html was {html}");
    assert!(html.contains("Laboratorio 1"), "html was {html}");
    assert!(html.contains("New"), "html was {html}");
    assert!(html.contains("Pending"), "html was {html}");
    assert!(html.contains("Nov 24, 12:00 PM"), "html was {html}");
    assert!(html.contains("Lectura"), "html was {html}");
}

#[test]
fn an_unknown_status_falls_back_to_no_data() {
    let html = common::with_locale(Language::En, || render(detail(), None));

    // The status and the missing due date both localize to the same note.
    assert_eq!(html.matches("No date").count(), 2, "html was {html}");
}

#[test]
fn resources_have_an_open_action() {
    let html = common::with_locale(Language::En, || render(detail(), None));

    assert!(html.contains("Resources"), "html was {html}");
    assert!(html.contains("Guia.pdf"), "html was {html}");
    assert_eq!(html.matches(">Open<").count(), 1, "html was {html}");
}

#[test]
fn grades_show_the_source_and_the_formatted_value() {
    let html = common::with_locale(Language::En, || render(detail(), None));

    assert!(html.contains("Grades"), "html was {html}");
    assert!(html.contains("Control 1"), "html was {html}");
    assert!(html.contains("banner"), "html was {html}");
    assert!(html.contains("uppercase"), "html was {html}");
    assert!(html.contains("6.5"), "html was {html}");
}

#[test]
fn schedule_rows_fall_back_on_the_title_and_the_room() {
    let html = common::with_locale(Language::En, || render(detail(), None));

    assert!(html.contains("Schedule"), "html was {html}");
    assert!(
        html.contains("Catedra · 08:30 - 09:50 · B-12"),
        "html was {html}"
    );
    assert!(
        html.contains("Class · 10:00 - 11:20 · No room"),
        "html was {html}"
    );
}

#[test]
fn empty_lists_use_their_notes() {
    let html = common::with_locale(Language::En, || render(empty_detail(), None));

    for note in [
        "No tasks recorded.",
        "No resources downloaded.",
        "No grades published.",
        "No schedule blocks.",
    ] {
        assert!(html.contains(note), "{note} missing in {html}");
    }
}

#[test]
fn a_focused_task_and_resource_are_marked_current() {
    let task = common::with_locale(Language::En, || render(detail(), Some("a1")));
    assert!(task.contains("id=\"focus-a1\""), "html was {task}");
    assert!(task.contains("aria-current=\"true\""), "html was {task}");
    assert_eq!(
        task.matches("aria-current=\"true\"").count(),
        1,
        "html was {task}"
    );
    assert!(task.contains("bg-primary/5"), "html was {task}");

    let resource = common::with_locale(Language::En, || render(detail(), Some("f1")));
    assert!(resource.contains("id=\"focus-f1\""), "html was {resource}");
    assert_eq!(
        resource.matches("aria-current=\"true\"").count(),
        1,
        "html was {resource}"
    );
}

#[test]
fn the_fallbacks_and_the_grade_follow_the_locale() {
    let html = common::with_locale(Language::Es, || render(detail(), None));

    assert!(html.contains("Secciones"), "html was {html}");
    assert!(
        html.contains("Sin sección · Sin profesor"),
        "html was {html}"
    );
    assert!(html.contains("Notas"), "html was {html}");
    assert!(html.contains("6,5"), "html was {html}");
    assert!(!html.contains("6.5"), "html was {html}");
}

#[test]
fn course_detail_route_round_trips_with_and_without_focus() {
    for focus in [None, Some("a1".to_string())] {
        let route = Route::CourseDetail {
            group_id: "g1".into(),
            focus,
        };
        let text = route.to_string();
        let parsed = Route::from_str(&text).expect("a valid route");

        assert_eq!(parsed, route, "route was {text}");
    }
}
