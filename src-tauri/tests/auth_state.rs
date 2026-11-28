use rust_app_lib::auth::{SessionState, Source};

fn url(value: &str) -> url::Url {
    url::Url::parse(value).unwrap()
}

#[test]
fn one_failed_source_leaves_session_partial() {
    let mut state = SessionState::Disconnected;
    state = state.connecting(Source::Moodle.into());
    state = state.source_ok(Source::Moodle.into());
    state = state.source_failed(Source::Banner.into());
    assert_eq!(state, SessionState::Partial);
}

#[test]
fn healthy_source_does_not_recover_partial_session() {
    assert_eq!(
        SessionState::Partial.source_ok(Source::Banner.into()),
        SessionState::Partial
    );
}

#[test]
fn final_state_depends_on_failed_sources() {
    assert_eq!(SessionState::outcome(0), SessionState::Connected);
    assert_eq!(SessionState::outcome(1), SessionState::Partial);
    assert_eq!(SessionState::outcome(3), SessionState::Partial);
}

#[test]
fn all_sources_healthy_end_connected() {
    let mut state = SessionState::Disconnected;
    for source in Source::ALL {
        state = state.connecting(source.into()).source_ok(source.into());
    }
    assert_eq!(state, SessionState::Connected);
}

#[test]
fn connection_order_is_ellucian_moodle_banner() {
    assert_eq!(
        Source::ALL,
        [Source::Ellucian, Source::Moodle, Source::Banner]
    );
}

#[test]
fn wire_source_mirrors_every_auth_source() {
    for source in Source::ALL {
        assert_eq!(
            serde_json::to_value(contracts::Source::from(source)).unwrap(),
            source.as_str()
        );
    }
}

#[test]
fn session_serializes_for_the_interface() {
    let json = serde_json::to_value(SessionState::Connecting(Source::Moodle.into())).unwrap();
    assert_eq!(json["state"], "connecting");
    assert_eq!(json["source"], "moodle");

    let json = serde_json::to_value(SessionState::Connected).unwrap();
    assert_eq!(json["state"], "connected");
    assert!(json.get("source").is_none());
}

#[test]
fn recognizes_final_page_of_each_source() {
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
