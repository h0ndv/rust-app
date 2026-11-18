use rust_app_lib::auth::{SessionState, Source};

fn url(value: &str) -> url::Url {
    url::Url::parse(value).unwrap()
}

#[test]
fn una_fuente_caida_deja_la_sesion_en_parcial() {
    let mut state = SessionState::Disconnected;
    state = state.connecting(Source::Moodle);
    state = state.source_ok(Source::Moodle);
    state = state.source_failed(Source::Banner);
    assert_eq!(state, SessionState::Partial);
}

#[test]
fn una_fuente_sana_no_recupera_la_sesion_parcial() {
    assert_eq!(
        SessionState::Partial.source_ok(Source::Banner),
        SessionState::Partial
    );
}

#[test]
fn el_estado_final_depende_de_las_fuentes_caidas() {
    assert_eq!(SessionState::outcome(0), SessionState::Connected);
    assert_eq!(SessionState::outcome(1), SessionState::Partial);
    assert_eq!(SessionState::outcome(3), SessionState::Partial);
}

#[test]
fn las_tres_fuentes_bien_terminan_conectadas() {
    let mut state = SessionState::Disconnected;
    for source in Source::ALL {
        state = state.connecting(source).source_ok(source);
    }
    assert_eq!(state, SessionState::Connected);
}

#[test]
fn el_orden_de_conexion_es_ellucian_moodle_banner() {
    assert_eq!(
        Source::ALL,
        [Source::Ellucian, Source::Moodle, Source::Banner]
    );
}

#[test]
fn la_sesion_se_serializa_para_la_interfaz() {
    let json = serde_json::to_value(SessionState::Connecting(Source::Moodle)).unwrap();
    assert_eq!(json["state"], "connecting");
    assert_eq!(json["source"], "moodle");

    let json = serde_json::to_value(SessionState::Connected).unwrap();
    assert_eq!(json["state"], "connected");
    assert!(json.get("source").is_none());
}

#[test]
fn reconoce_la_pagina_final_de_cada_fuente() {
    assert!(Source::Ellucian.is_ready(&url("https://experience.elluciancloud.com/ustc/")));
    assert!(!Source::Ellucian.is_ready(&url("https://experience.elluciancloud.com/ustc/auth")));
    assert!(!Source::Ellucian.is_ready(&url("https://portal.santotomas.cl/")));

    assert!(Source::Moodle.is_ready(&url("https://aulasvirtuales.santotomas.cl/my/")));
    assert!(!Source::Moodle.is_ready(&url("https://aulasvirtuales.santotomas.cl/login/index.php")));

    assert!(Source::Banner.is_ready(&url(
        "https://selferviceprd.santotomas.cl/StudentSelfService/ssb/studentGrades"
    )));
    assert!(!Source::Banner.is_ready(&url("https://logineis.santotomas.cl/cas/login")));
}
