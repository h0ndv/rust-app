#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::str::FromStr;

use contracts::{RecoveryNoticeDto, SessionState, Source};
use dioxus::prelude::*;
use rust_app_ui::components::{LanguageSelect, RecoveryBanner, SessionBanner, SyncBadge};
use rust_app_ui::domain::SyncState;
use rust_app_ui::i18n::Language;
use rust_app_ui::router::Route;

#[component]
fn SessionBannerHarness(state: Option<SessionState>, connecting: bool) -> Element {
    rsx! {
        SessionBanner { state, connecting, on_connect: |_| {} }
    }
}

#[component]
fn LanguageSelectHarness(value: Language) -> Element {
    rsx! {
        LanguageSelect { value, on_change: |_| {} }
    }
}

#[component]
fn RecoveryBannerHarness(notice: RecoveryNoticeDto) -> Element {
    rsx! {
        RecoveryBanner { notice, on_dismiss: |_| {} }
    }
}

#[test]
fn route_round_trips_course_detail_with_focus() {
    let route = Route::from_str("/ramos/abc?focus=task-1").unwrap();
    assert_eq!(
        route,
        Route::CourseDetail {
            group_id: "abc".into(),
            focus: Some("task-1".into())
        }
    );
    assert_eq!(route.to_string(), "/ramos/abc?focus=task-1");
}

#[test]
fn every_top_level_route_round_trips_its_path() {
    let routes = [
        (Route::Home {}, "/"),
        (Route::Courses {}, "/ramos"),
        (Route::Tasks {}, "/tareas"),
        (Route::Resources {}, "/recursos"),
        (Route::Grades {}, "/notas"),
        (Route::Schedule {}, "/horario"),
        (Route::News {}, "/novedades"),
        (Route::Search {}, "/buscar"),
        (Route::Settings {}, "/ajustes"),
    ];

    for (route, path) in routes {
        assert_eq!(route.to_string(), path);
        assert_eq!(Route::from_str(path).unwrap(), route);
    }
}

#[test]
fn course_detail_without_focus_round_trips() {
    let route = Route::from_str("/ramos/abc").unwrap();
    assert_eq!(
        route,
        Route::CourseDetail {
            group_id: "abc".into(),
            focus: None
        }
    );
    // Dioxus writes the query marker even when the optional argument is absent.
    assert_eq!(route.to_string().trim_end_matches('?'), "/ramos/abc");
}

#[test]
fn session_banner_shows_the_expired_source() {
    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! {
            SessionBannerHarness { state: Some(SessionState::Expired(Source::Moodle)), connecting: false }
        })
    });

    assert!(html.contains("Session expired"), "html was {html}");
    assert!(html.contains("Moodle"), "html was {html}");
    assert!(html.contains("Reconnect"), "html was {html}");
}

#[test]
fn session_banner_offers_connect_without_a_session() {
    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! {
            SessionBannerHarness { state: Some(SessionState::Disconnected), connecting: false }
        })
    });

    assert!(html.contains("No session"), "html was {html}");
    assert!(html.contains("Connect"), "html was {html}");
}

#[test]
fn session_banner_hides_when_connected() {
    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! {
            SessionBannerHarness { state: Some(SessionState::Connected), connecting: false }
        })
    });

    assert!(!html.contains("Session expired"), "html was {html}");
    assert!(!html.contains("No session"), "html was {html}");
}

#[test]
fn sync_badge_shows_running() {
    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! { SyncBadge { state: SyncState::Running, progress: 42 } })
    });

    assert!(html.contains("42"), "html was {html}");
    assert!(html.contains("Syncing"), "html was {html}");
}

#[test]
fn sync_badge_shows_the_idle_label() {
    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! { SyncBadge { state: SyncState::Idle, progress: 0 } })
    });

    assert!(html.contains("Not synced yet"), "html was {html}");
}

#[test]
fn language_select_lists_both_languages() {
    let html = dioxus_ssr::render_element(rsx! {
        LanguageSelectHarness { value: Language::En }
    });

    assert!(html.contains("English"), "html was {html}");
    assert!(html.contains("Español"), "html was {html}");
    assert!(html.contains("value=\"en\""), "html was {html}");
}

#[test]
fn recovery_banner_shows_the_backup_path_and_dismisses() {
    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! {
            RecoveryBannerHarness {
                notice: RecoveryNoticeDto {
                    backup_path: "C:\\data\\panel.db.corrupt-1".into(),
                    reason: "not a database".into(),
                },
            }
        })
    });

    assert!(html.contains("panel.db.corrupt-1"), "html was {html}");
    assert!(html.contains("Dismiss"), "html was {html}");
}
