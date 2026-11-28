#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::{DashboardDto, SessionState, SettingsDto, Source, TaskDto};
use rust_app_ui::app::{Mode, startup_mode};
use rust_app_ui::i18n::Language;
use rust_app_ui::notifications::should_notify;

fn task(id: &str) -> TaskDto {
    TaskDto {
        id: id.into(),
        course_id: "moodle-1".into(),
        course_name: "Data Mining".into(),
        title: "Lab".into(),
        ..Default::default()
    }
}

#[test]
fn startup_prefers_the_app_when_there_is_local_data() {
    assert_eq!(startup_mode(true, &SessionState::Disconnected), Mode::App);
    assert_eq!(
        startup_mode(false, &SessionState::Disconnected),
        Mode::Onboarding
    );
}

#[test]
fn startup_opens_the_app_for_any_live_session() {
    for session in [
        SessionState::Connected,
        SessionState::Partial,
        SessionState::Connecting(Source::Moodle),
        SessionState::Expired(Source::Banner),
    ] {
        assert_eq!(startup_mode(false, &session), Mode::App, "{session:?}");
    }
}

#[test]
fn startup_with_data_wins_over_an_expired_session() {
    assert_eq!(
        startup_mode(true, &SessionState::Expired(Source::Ellucian)),
        Mode::App
    );
}

#[test]
fn notifications_only_fire_for_overdue_or_due_soon() {
    common::with_locale(Language::En, || {
        let settings = SettingsDto {
            notifications: true,
            ..Default::default()
        };
        let dashboard = DashboardDto {
            overdue: vec![task("a")],
            ..Default::default()
        };

        let (title, body) = should_notify(&settings, &dashboard).unwrap();
        assert_eq!(title, "Pending work");
        assert_eq!(body, "1 overdue task");
    });
}

#[test]
fn notifications_join_both_counts_with_a_middle_dot() {
    common::with_locale(Language::En, || {
        let settings = SettingsDto::default();
        let dashboard = DashboardDto {
            overdue: vec![task("a"), task("b")],
            due_soon: vec![task("c")],
            ..Default::default()
        };

        let (_, body) = should_notify(&settings, &dashboard).unwrap();
        assert_eq!(body, "2 overdue tasks · 1 task due soon");
    });
}

#[test]
fn notifications_stay_quiet_when_the_toggle_is_off() {
    let settings = SettingsDto {
        notifications: false,
        ..Default::default()
    };
    let dashboard = DashboardDto {
        overdue: vec![task("a")],
        due_soon: vec![task("b")],
        ..Default::default()
    };

    assert_eq!(should_notify(&settings, &dashboard), None);
}

#[test]
fn notifications_stay_quiet_without_pending_work() {
    let settings = SettingsDto::default();

    assert_eq!(should_notify(&settings, &DashboardDto::default()), None);
}
