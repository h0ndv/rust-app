#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::{GradeRowDto, GradeTotalDto, GradesDto};
use dioxus::prelude::*;
use rust_app_ui::features::grades::GradesView;
use rust_app_ui::i18n::Language;

fn fixture() -> GradesDto {
    GradesDto {
        term: "202620".into(),
        terms: vec!["202620".into()],
        banner: vec![GradeRowDto {
            course: "Minería de Datos".into(),
            name: "Minería De Datos".into(),
            grade: Some(6.5),
            max_grade: Some(7.0),
            source: "banner".into(),
        }],
        moodle_totals: vec![GradeTotalDto {
            course: "Minería de Datos".into(),
            total: Some(5.5),
            max_total: Some(7.0),
            source: "moodle".into(),
        }],
        gpa: None,
    }
}

fn empty() -> GradesDto {
    GradesDto {
        term: "202620".into(),
        terms: vec!["202620".into()],
        ..Default::default()
    }
}

fn render(grades: GradesDto) -> String {
    dioxus_ssr::render_element(rsx! {
        GradesView { grades }
    })
}

#[test]
fn shows_banner_grades_and_moodle_total_separately() {
    let html = common::with_locale(Language::En, || render(fixture()));

    assert!(html.contains("Official grade"), "html was {html}");
    assert!(html.contains("Moodle total"), "html was {html}");
    assert!(html.contains("6.5"), "html was {html}");
    assert!(html.contains("5.5"), "html was {html}");
    assert!(html.contains("Grade"), "html was {html}");
    assert_eq!(
        html.matches("Minería de Datos").count(),
        2,
        "html was {html}"
    );
}

#[test]
fn says_when_a_total_is_missing() {
    let html = common::with_locale(Language::En, || render(empty())).to_lowercase();

    assert_eq!(html.matches("not published").count(), 2, "html was {html}");
}

#[test]
fn shows_the_term_gpa_only_when_present() {
    let without = common::with_locale(Language::En, || render(fixture()));
    assert!(!without.contains("Term GPA"), "html was {without}");

    let mut grades = fixture();
    grades.gpa = Some(6.2);
    let html = common::with_locale(Language::En, || render(grades));
    assert!(html.contains("Term GPA: 6.2"), "html was {html}");
}

#[test]
fn grades_follow_the_locale_number_format() {
    let html = common::with_locale(Language::Es, || render(fixture()));

    assert!(html.contains("Nota oficial"), "html was {html}");
    assert!(html.contains("Total Moodle"), "html was {html}");
    assert!(html.contains("6,5"), "html was {html}");
    assert!(html.contains("5,5"), "html was {html}");
    assert!(!html.contains("6.5"), "html was {html}");
}

#[test]
fn the_empty_note_follows_the_locale() {
    let html = common::with_locale(Language::Es, || render(empty())).to_lowercase();

    assert_eq!(html.matches("sin publicar").count(), 2, "html was {html}");
}
