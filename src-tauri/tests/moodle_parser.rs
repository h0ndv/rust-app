use rust_app_lib::error::AppError;
use rust_app_lib::moodle::parser::{
    parse_announcements_html, parse_assignment_html, parse_calendar_json, parse_course_html,
    parse_courses_json, parse_grade_total_html,
};

#[test]
fn parses_eight_courses() {
    let json = include_str!("fixtures/moodle/courses.json");
    let courses = parse_courses_json(json).unwrap();
    assert_eq!(courses.len(), 8);
    assert!(
        courses
            .iter()
            .any(|c| c.fullname.contains("Minería de Datos"))
    );
}

#[test]
fn parses_course_calendar() {
    let json = include_str!("fixtures/moodle/calendar.json");
    let events = parse_calendar_json(json).unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.modulename == "assign"));
    assert!(events.iter().all(|e| e.timesort > 0));
    assert!(events.iter().any(|e| e.course_id > 0));
}

#[test]
fn parses_section_with_activities() {
    let html = include_str!("fixtures/moodle/course_821065_s3.html");
    let section = parse_course_html(html, 3).unwrap();

    assert_eq!(section.name, "Unidad 2");
    assert_eq!(section.activities.len(), 8);

    // `label` items are content blocks with no link of their own: the invariant
    // of name and URL only applies to linkable activities.
    let linkable: Vec<_> = section
        .activities
        .iter()
        .filter(|activity| activity.modtype != "label")
        .collect();
    assert_eq!(linkable.len(), 4);
    assert!(
        linkable
            .iter()
            .all(|a| a.cmid > 0 && !a.name.is_empty() && a.url.contains("/mod/"))
    );
    assert!(linkable.iter().any(|a| a.name == "Ejercicio 2.4"));
    assert!(linkable.iter().all(|a| a.modtype == "assign"));
}

#[test]
fn empty_html_is_parse_error() {
    assert!(matches!(parse_course_html("", 0), Err(AppError::Parse(_))));
}

#[test]
fn parses_assignment_state_without_submission() {
    let html = include_str!("fixtures/moodle/assign_821065_module-22053263.html");
    let assignment = parse_assignment_html(html).unwrap();
    assert!(!assignment.submitted);
    assert!(!assignment.graded);
    assert!(assignment.status_raw.contains("envíos"));
}

#[test]
fn parses_forum_announcements() {
    let html = r#"<div class="forumpost"><h3>Cambio de sala</h3><div class="posting">La clase se mueve a la 505.</div></div>"#;
    let items = parse_announcements_html(html).unwrap();
    assert_eq!(items[0].title, "Cambio de sala");
    assert_eq!(items[0].body, "La clase se mueve a la 505.");
}

#[test]
fn parses_moodle_total() {
    let html = r#"<table class="user-grade"><tr class="course-total"><td>Total</td><td>6,2</td></tr></table>"#;
    let total = parse_grade_total_html(html).unwrap();
    assert_eq!(total.total, 6.2);
}

#[test]
fn login_page_is_expired_session() {
    let json =
        r#"[{"error":true,"exception":{"errorcode":"requirelogin","message":"Sesión vencida"}}]"#;
    assert!(matches!(
        parse_courses_json(json),
        Err(AppError::Unauthorized)
    ));
}
