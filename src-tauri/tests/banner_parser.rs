use rust_app_lib::banner::parser::{
    parse_courses_response, parse_gpa_response, parse_options_response,
};
use rust_app_lib::error::AppError;

#[test]
fn parsea_notas_del_periodo() {
    let json = include_str!("fixtures/banner/courses.json");
    let grades = parse_courses_response(json).unwrap();

    assert_eq!(grades.len(), 8);
    assert!(grades.iter().all(|g| !g.crn.is_empty()));
    assert!(
        grades
            .iter()
            .any(|g| g.crn == "30061" && g.section == "00G")
    );
    assert!(
        grades
            .iter()
            .any(|g| g.subject == "IGR" && g.number == "00167"),
        "Minería de Datos aparece en cátedra y laboratorio"
    );
    // Today no grade is published: all fields arrive null.
    assert!(grades.iter().all(|g| g.final_grade.is_none()));
}

#[test]
fn lee_una_nota_publicada_y_otra_pendiente() {
    let json = r#"{"success":true,"data":[
        {"subjectCode":"IGR","courseNumber":"00167","courseReferenceNumber":"30061",
         "section":"00G","courseTitle":"Minería De Datos","finalGrade":"6,5",
         "midtermGrade":null,"historyFinalGrade":null,"calculatedFinalGrade":null,
         "termCode":"202620","termDescription":"Semestre Pregrado 2026 - 2","levelCode":"02"},
        {"subjectCode":"IGR","courseNumber":"00167","courseReferenceNumber":"30060",
         "section":"A0C","courseTitle":"Minería De Datos","finalGrade":null,
         "midtermGrade":"5,8","historyFinalGrade":null,"calculatedFinalGrade":null,
         "termCode":"202620","termDescription":"Semestre Pregrado 2026 - 2","levelCode":"02"}
    ]}"#;

    let grades = parse_courses_response(json).unwrap();
    assert_eq!(grades[0].final_grade.as_deref(), Some("6,5"));
    assert!(grades.iter().any(|g| g.final_grade.is_none()));
    assert_eq!(grades[1].midterm_grade.as_deref(), Some("5,8"));
}

#[test]
fn html_de_sso_es_unauthorized() {
    let html = "<!DOCTYPE html><html><head><title>Redirecting</title><meta name=\"PageID\" content=\"BssoInterrupt\"></head></html>";
    assert!(matches!(
        parse_courses_response(html),
        Err(AppError::Unauthorized)
    ));
}

#[test]
fn una_respuesta_sin_datos_es_error_de_parseo() {
    assert!(matches!(
        parse_courses_response(r#"{"success":false,"message":"sesión"}"#),
        Err(AppError::Parse(_))
    ));
}

#[test]
fn parsea_los_periodos() {
    let json = include_str!("fixtures/banner/terms.json");
    let terms = parse_options_response(json).unwrap();

    assert!(terms.len() >= 6);
    assert_eq!(terms[0].code, "-1");
    assert!(terms.iter().any(|t| t.code == "202620"));
    assert!(terms.iter().all(|t| !t.description.is_empty()));
}

#[test]
fn parsea_niveles_y_planes() {
    let levels = parse_options_response(include_str!("fixtures/banner/level.json")).unwrap();
    assert_eq!(levels[0].code, "02");
    assert_eq!(levels[0].description, "Profesional");

    let paths = parse_options_response(include_str!("fixtures/banner/study_path.json")).unwrap();
    assert!(paths.iter().any(|p| p.code == "-2"));
}

#[test]
fn parsea_el_gpa_del_periodo() {
    let json = include_str!("fixtures/banner/gpa.json");
    let gpa = parse_gpa_response(json).unwrap();

    // With no published grades, the GPA arrives null but the object exists.
    assert!(gpa.selected_term.is_none());
    assert!(gpa.cumulative.is_none());
}

#[test]
fn lee_un_gpa_numerico() {
    let json = r#"{"selectedTerm":{"gpa":5.42},"cumulative":{"gpa":5.11}}"#;
    let gpa = parse_gpa_response(json).unwrap();
    assert_eq!(gpa.selected_term, Some(5.42));
    assert_eq!(gpa.cumulative, Some(5.11));
}
