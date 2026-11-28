#![cfg(not(target_arch = "wasm32"))]

mod common;

use rust_app_ui::format::{format_date, format_grade, format_size, weekdays};
use rust_app_ui::i18n::Language;

#[test]
fn chilean_grade_uses_a_comma() {
    assert_eq!(format_grade(Some(6.5), Language::Es), "6,5");
}

#[test]
fn english_grade_uses_a_dot() {
    assert_eq!(format_grade(Some(6.5), Language::En), "6.5");
    assert_eq!(format_grade(Some(7.0), Language::En), "7.0");
}

#[test]
fn grades_round_half_away_from_zero() {
    assert_eq!(format_grade(Some(6.25), Language::Es), "6,3");
    assert_eq!(format_grade(Some(5.25), Language::En), "5.3");
}

#[test]
fn missing_grade_uses_the_catalog() {
    common::with_locale(Language::Es, || {
        assert_eq!(format_grade(None, Language::Es), "Sin publicar");
    });
}

#[test]
fn chilean_date_uses_the_offset() {
    // 2026-11-24T12:00:00Z at UTC-3 must render as local 09:00 on a 12-hour clock.
    let formatted = format_date(Some(1_795_521_600), Language::Es, -180);
    assert_eq!(formatted, "24-nov, 09:00 a. m.");
}

#[test]
fn chilean_afternoon_uses_the_pm_marker() {
    // 2026-11-24T20:00:00Z at UTC-3 is local 17:00.
    let formatted = format_date(Some(1_795_550_400), Language::Es, -180);
    assert_eq!(formatted, "24-nov, 05:00 p. m.");
}

#[test]
fn english_date_uses_the_offset() {
    let formatted = format_date(Some(1_795_521_600), Language::En, -180);
    assert_eq!(formatted, "Nov 24, 09:00 AM");
}

#[test]
fn missing_date_uses_the_catalog() {
    common::with_locale(Language::En, || {
        assert_eq!(format_date(None, Language::En, -180), "No date");
    });
}

#[test]
fn sizes_are_scaled_and_localized() {
    assert_eq!(format_size(Some(1024), Language::En), "1 KB");
    assert_eq!(format_size(Some(1536), Language::Es), "1,5 KB");
    assert_eq!(format_size(Some(500), Language::En), "500 B");
    assert_eq!(format_size(Some(2 * 1024 * 1024), Language::En), "2 MB");
    assert_eq!(format_size(Some(0), Language::Es), "");
    assert_eq!(format_size(None, Language::En), "");
}

#[test]
fn sizes_round_half_away_from_zero() {
    // 1280 / 1024 is exactly 1.25 KB.
    assert_eq!(format_size(Some(1280), Language::En), "1.3 KB");
}

#[test]
fn weekdays_start_on_monday() {
    let english = weekdays(Language::En);
    let spanish = weekdays(Language::Es);

    assert_eq!(english.len(), 7);
    assert_eq!(english[0], "Monday");
    assert_eq!(english[6], "Sunday");
    assert_eq!(spanish[0].to_lowercase(), "lunes");
    assert_eq!(spanish[6].to_lowercase(), "domingo");
}
