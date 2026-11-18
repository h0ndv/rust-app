use rust_app_lib::ellucian::models::AcademicPeriod;
use rust_app_lib::ellucian::parser::{
    current_period, moodle_shortname, parse_classes_response, parse_periods_response,
    parse_schedule_response,
};
use serde_json::json;

#[test]
fn parsea_eventos_con_sala_y_profesor() {
    let json = include_str!("fixtures/ellucian/schedule.json");
    let events = parse_schedule_response(json).unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.room.is_some()));
}

#[test]
fn el_evento_trae_dia_hora_sala_y_profesor() {
    let events = parse_schedule_response(include_str!("fixtures/ellucian/schedule.json")).unwrap();
    let event = &events[0];

    assert_eq!(event.day_of_week, 5, "viernes");
    assert_eq!(event.start_time, "19:21");
    assert_eq!(event.end_time, "20:00");
    assert_eq!(event.room.as_deref(), Some("Sala 401"));
    assert_eq!(event.building.as_deref(), Some("Dieciocho N° 1191"));
    assert_eq!(
        event.instructor.as_deref(),
        Some("PABLO LILLO GALLARDO"),
        "el apellido compuesto viene separado por punto y coma"
    );
}

#[test]
fn parsea_las_ocho_asignaturas() {
    let classes = parse_classes_response(include_str!("fixtures/ellucian/classes.json")).unwrap();

    assert_eq!(classes.len(), 8);
    assert!(classes.iter().all(|c| !c.crn.is_empty()));
    assert!(classes.iter().all(|c| c.term == "202620"));
    assert!(
        classes
            .iter()
            .any(|c| c.subject == "IGR" && c.number == "00167")
    );
    assert!(
        classes
            .iter()
            .any(|c| c.title == "Creatividad en la Innovación y Emprendimiento")
    );
    assert!(classes.iter().all(|c| c.moodle_shortname.is_none()));
}

#[test]
fn elige_el_periodo_mas_reciente_comenzado() {
    let periods = parse_periods_response(include_str!("fixtures/ellucian/periods.json")).unwrap();
    assert_eq!(periods.len(), 6, "se deduplican por id");

    let now = chrono::DateTime::parse_from_rfc3339("2026-11-17T00:00:00Z")
        .unwrap()
        .timestamp();
    let current = current_period(&periods, now).unwrap();
    assert_eq!(current.title, "Semestre Pregrado 2026 - 2");
}

#[test]
fn sin_periodos_comenzados_toma_el_primero() {
    let periods = vec![
        AcademicPeriod {
            id: "b".into(),
            title: "2027 - 1".into(),
            start_on: Some(
                chrono::DateTime::parse_from_rfc3339("2027-03-01T00:00:00Z")
                    .unwrap()
                    .timestamp(),
            ),
        },
        AcademicPeriod {
            id: "a".into(),
            title: "2027 - 2".into(),
            start_on: Some(
                chrono::DateTime::parse_from_rfc3339("2027-08-01T00:00:00Z")
                    .unwrap()
                    .timestamp(),
            ),
        },
    ];

    let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .timestamp();
    let current = current_period(&periods, now).unwrap();
    assert_eq!(current.id, "b");
}

#[test]
fn encuentra_el_shortname_de_moodle_en_un_enlace() {
    let node = json!({
        "aulaVirtual": "https://aulasvirtuales.santotomas.cl/course/view.php?id=821065",
        "shortname": "30061.202620"
    });
    assert_eq!(moodle_shortname(&node).as_deref(), Some("30061.202620"));

    assert!(moodle_shortname(&json!({ "otro": "valor" })).is_none());
}

#[test]
fn una_consulta_inexistente_es_error_de_parseo() {
    let json = r#"{"errors":[{"code":"query-not-found","message":"Query not found for 'x'"}]}"#;
    assert!(matches!(
        parse_classes_response(json),
        Err(rust_app_lib::error::AppError::Parse(_))
    ));
}

/// The endpoint answers 400 if the descriptor does not use its exact names.
#[test]
fn el_descriptor_usa_los_nombres_del_endpoint() {
    use rust_app_lib::ellucian::queries::{
        CARD_ID, EXTENSION_ID, EthosQuery, MANIFEST_TYPE, SCHEDULE_QUERY,
    };

    let body = serde_json::to_value(EthosQuery::new(
        SCHEDULE_QUERY,
        json!({ "sectionId": "abc" }),
    ))
    .unwrap();

    assert_eq!(body["extensionId"], EXTENSION_ID);
    assert_eq!(body["cardId"], CARD_ID);
    assert_eq!(body["manifestType"], MANIFEST_TYPE);
    assert_eq!(body["queryId"], "instructional-events-by-section");
    assert_eq!(body["properties"]["sectionId"], "abc");
    assert!(body.get("extension_id").is_none());
}
