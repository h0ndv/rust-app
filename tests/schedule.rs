#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::ScheduleEventDto;
use dioxus::prelude::*;
use rust_app_ui::features::schedule::ScheduleView;
use rust_app_ui::i18n::Language;

fn block(
    id: &str,
    day: i64,
    title: &str,
    start: &str,
    end: &str,
    room: Option<&str>,
    instructor: Option<&str>,
) -> ScheduleEventDto {
    ScheduleEventDto {
        id: id.into(),
        title: title.into(),
        day: Some(day),
        start: Some(start.into()),
        end: Some(end.into()),
        room: room.map(str::to_string),
        instructor: instructor.map(str::to_string),
        ..Default::default()
    }
}

fn render(events: Vec<ScheduleEventDto>) -> String {
    dioxus_ssr::render_element(rsx! {
        ScheduleView { events }
    })
}

fn position(html: &str, needle: &str) -> usize {
    html.find(needle)
        .unwrap_or_else(|| panic!("{needle} missing in {html}"))
}

#[test]
fn renders_seven_day_columns_and_empty_state() {
    let html = common::with_locale(Language::En, || {
        render(vec![block(
            "calc", 1, "Calculus", "08:00", "09:30", None, None,
        )])
    });

    assert!(html.contains("Monday"), "html was {html}");
    assert!(html.contains("08:00"), "html was {html}");
    assert!(html.contains("No classes"), "html was {html}");
    assert_eq!(html.matches("<section").count(), 7, "html was {html}");
    assert!(html.contains("md:grid-cols-2"), "html was {html}");
    assert!(html.contains("xl:grid-cols-7"), "html was {html}");
    assert_eq!(html.matches("No classes").count(), 6, "html was {html}");
}

#[test]
fn weekdays_start_on_monday() {
    let html = common::with_locale(Language::En, || {
        render(vec![block("a", 1, "A", "08:00", "09:00", None, None)])
    });

    let monday = position(&html, ">Monday<");
    let tuesday = position(&html, ">Tuesday<");
    let sunday = position(&html, ">Sunday<");
    assert!(monday < tuesday && tuesday < sunday, "html was {html}");
}

#[test]
fn blocks_stay_in_their_day_and_sort_by_start() {
    let html = common::with_locale(Language::En, || {
        render(vec![
            block("thu", 4, "thu", "09:00", "10:00", None, None),
            block("wed-late", 3, "wed-late", "11:00", "12:00", None, None),
            block("wed-early", 3, "wed-early", "08:00", "09:00", None, None),
            block("sun", 7, "sun", "17:00", "18:00", None, None),
        ])
    });

    let wednesday = position(&html, ">Wednesday<");
    let thursday = position(&html, ">Thursday<");
    let sunday = position(&html, ">Sunday<");

    let early = position(&html, "wed-early");
    let late = position(&html, "wed-late");
    assert!(
        wednesday < early && early < late && late < thursday,
        "html was {html}"
    );

    let thu = position(&html, ">thu<");
    assert!(
        thursday < thu && thu < position(&html, ">Friday<"),
        "html was {html}"
    );

    let sun = position(&html, ">sun<");
    assert!(sunday < sun, "html was {html}");

    assert_eq!(html.matches(">wed-early<").count(), 1, "html was {html}");
    assert_eq!(html.matches(">thu<").count(), 1, "html was {html}");
}

#[test]
fn blocks_with_the_same_start_keep_their_order() {
    let html = common::with_locale(Language::En, || {
        render(vec![
            block("first", 2, "first", "08:00", "09:00", None, None),
            block("second", 2, "second", "08:00", "09:00", None, None),
        ])
    });

    assert!(
        position(&html, ">first<") < position(&html, ">second<"),
        "html was {html}"
    );
}

#[test]
fn a_missing_start_sorts_first_and_unknown_days_are_dropped() {
    let no_start = ScheduleEventDto {
        id: "no-start".into(),
        title: "no-start".into(),
        day: Some(3),
        ..Default::default()
    };
    let no_day = ScheduleEventDto {
        id: "no-day".into(),
        title: "no-day".into(),
        start: Some("01:00".into()),
        ..Default::default()
    };
    let eighth = ScheduleEventDto {
        id: "eighth".into(),
        title: "eighth".into(),
        day: Some(8),
        start: Some("01:00".into()),
        ..Default::default()
    };

    let html = common::with_locale(Language::En, || {
        render(vec![
            block("morning", 3, "morning", "08:00", "09:00", None, None),
            no_start,
            no_day,
            eighth,
        ])
    });

    assert!(
        position(&html, ">no-start<") < position(&html, ">morning<"),
        "html was {html}"
    );
    assert!(!html.contains("no-day"), "html was {html}");
    assert!(!html.contains("eighth"), "html was {html}");
    assert_eq!(html.matches("<article").count(), 2, "html was {html}");
}

#[test]
fn shows_room_and_instructor_only_when_present() {
    let html = common::with_locale(Language::En, || {
        render(vec![
            block(
                "full",
                2,
                "Algebra",
                "08:00",
                "09:30",
                Some("B-12"),
                Some("Ada Lovelace"),
            ),
            block("bare", 2, "bare", "10:00", "11:00", None, None),
        ])
    });

    assert!(html.contains("B-12"), "html was {html}");
    assert!(html.contains("Ada Lovelace"), "html was {html}");
    assert_eq!(html.matches("No room").count(), 1, "html was {html}");
    assert_eq!(html.matches("B-12").count(), 1, "html was {html}");
    assert_eq!(html.matches("Ada Lovelace").count(), 1, "html was {html}");
}

#[test]
fn a_missing_title_start_end_and_room_fall_back() {
    let event = ScheduleEventDto {
        id: "sparse".into(),
        day: Some(5),
        ..Default::default()
    };

    let html = common::with_locale(Language::En, || render(vec![event]));

    assert!(html.contains(">Class<"), "html was {html}");
    assert!(html.contains(" - "), "html was {html}");
    assert!(html.contains("No room"), "html was {html}");
}

#[test]
fn the_empty_list_shows_the_empty_note() {
    let html = common::with_locale(Language::En, || render(Vec::new()));

    assert!(html.contains("No schedule synced yet."), "html was {html}");
    assert!(!html.contains("Monday"), "html was {html}");
    assert!(!html.contains("No classes"), "html was {html}");
}

#[test]
fn the_labels_and_copy_follow_the_locale() {
    let html = common::with_locale(Language::Es, || {
        render(vec![block("a", 1, "Cálculo", "08:00", "09:30", None, None)])
    });

    assert!(html.contains("lunes"), "html was {html}");
    assert!(html.contains("domingo"), "html was {html}");
    assert!(html.contains("Sin clases"), "html was {html}");
    assert!(html.contains("Sin sala"), "html was {html}");
    assert!(!html.contains("Monday"), "html was {html}");

    let empty = common::with_locale(Language::Es, || render(Vec::new()));
    assert!(
        empty.contains("no hay horario sincronizado"),
        "html was {empty}"
    );
}
