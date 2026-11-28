#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::{CourseGroupDto, CourseSectionDto};
use dioxus::prelude::*;
use rust_app_ui::features::courses::{CoursesGrid, course_route};
use rust_app_ui::i18n::Language;
use rust_app_ui::router::Route;

fn section(
    id: &str,
    section: Option<&str>,
    teacher: Option<&str>,
    fullname: &str,
    pending_tasks: i64,
    next_due_at: Option<i64>,
) -> CourseSectionDto {
    CourseSectionDto {
        id: id.into(),
        fullname: fullname.into(),
        section: section.map(str::to_string),
        teacher: teacher.map(str::to_string),
        pending_tasks,
        next_due_at,
        source: "moodle".into(),
        ..Default::default()
    }
}

fn group() -> CourseGroupDto {
    CourseGroupDto {
        id: "g1".into(),
        code: "IGR-001".into(),
        name: "Mineria de Datos".into(),
        sections: vec![
            section(
                "s1",
                Some("1"),
                Some("Ana Perez"),
                "Mineria de Datos - 1",
                3,
                Some(1_795_521_600),
            ),
            section("s2", None, None, "Mineria Virtual", 0, None),
        ],
    }
}

fn render(groups: Vec<CourseGroupDto>) -> String {
    dioxus_ssr::render_element(rsx! {
        CoursesGrid { groups }
    })
}

#[test]
fn course_route_targets_the_detail_screen_without_focus() {
    assert_eq!(
        course_route("g1"),
        Route::CourseDetail {
            group_id: "g1".into(),
            focus: None,
        }
    );
}

#[test]
fn an_empty_grid_shows_the_empty_note() {
    let html = common::with_locale(Language::En, || render(Vec::new()));

    assert!(html.contains("No courses synced yet."), "html was {html}");
    assert!(!html.contains("IGR-001"), "html was {html}");
}

#[test]
fn cards_show_the_code_name_and_each_section() {
    let html = common::with_locale(Language::En, || render(vec![group()]));

    assert!(html.contains("IGR-001"), "html was {html}");
    assert!(html.contains("Mineria de Datos"), "html was {html}");
    assert!(html.contains("Ana Perez"), "html was {html}");
    assert!(html.contains(">1<"), "html was {html}");
    assert!(html.contains(">Section<"), "html was {html}");
    assert!(html.contains("Mineria Virtual"), "html was {html}");
}

#[test]
fn pending_work_uses_the_plural_and_the_due_date() {
    let html = common::with_locale(Language::En, || render(vec![group()]));

    assert!(html.contains("3 pending tasks"), "html was {html}");
    assert!(html.contains("Nov 24, 12:00 PM"), "html was {html}");
    assert!(!html.contains("0 pending"), "html was {html}");
}

#[test]
fn a_single_pending_task_uses_the_singular_form() {
    let html = common::with_locale(Language::En, || {
        render(vec![CourseGroupDto {
            sections: vec![section("s1", Some("1"), None, "Mineria - 1", 1, None)],
            ..group()
        }])
    });

    assert!(html.contains("1 pending task"), "html was {html}");
    assert!(!html.contains("1 pending tasks"), "html was {html}");
}

#[test]
fn a_missing_teacher_falls_back_to_the_section_fullname() {
    let html = common::with_locale(Language::En, || {
        render(vec![CourseGroupDto {
            sections: vec![section("s1", Some("2"), None, "Mineria Virtual", 0, None)],
            ..group()
        }])
    });

    assert!(html.contains(">2<"), "html was {html}");
    assert!(html.contains("Mineria Virtual"), "html was {html}");
}

#[test]
fn the_empty_note_follows_the_locale() {
    let html = common::with_locale(Language::Es, || render(Vec::new()));

    assert!(
        html.contains("Todavía no hay ramos sincronizados."),
        "html was {html}"
    );
}
