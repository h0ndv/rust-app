#![cfg(not(target_arch = "wasm32"))]

use contracts::{Source, SyncRun, SyncStatus, SyncStatusKind};
use rust_app_ui::domain::SyncState;
use rust_app_ui::state::{SyncView, apply_finished, apply_progress};

#[test]
fn sync_view_starts_idle() {
    let view = SyncView::default();

    assert_eq!(view.state, SyncState::Idle);
    assert_eq!(view.progress, 0);
    assert_eq!(view.source, None);
    assert_eq!(view.requests, 0);
    assert!(view.errors.is_empty());
    assert!(view.last_run.is_none());
}

#[test]
fn progress_copies_every_field_of_the_status() {
    let mut view = SyncView::default();

    apply_progress(
        &mut view,
        SyncStatus {
            state: SyncStatusKind::Running,
            source: Some(Source::Moodle),
            progress: 42,
            requests: 17,
            errors: vec!["Moodle timed out".into()],
        },
    );

    assert_eq!(view.state, SyncState::Running);
    assert_eq!(view.source, Some(Source::Moodle));
    assert_eq!(view.progress, 42);
    assert_eq!(view.requests, 17);
    assert_eq!(view.errors, vec!["Moodle timed out".to_string()]);
    assert!(view.last_run.is_none());
}

#[test]
fn sync_finished_sets_progress_and_invalidates_data() {
    let mut view = SyncView::default();

    apply_finished(
        &mut view,
        SyncRun {
            id: "1".into(),
            status: SyncStatusKind::Ok,
            requests: 12,
            started_at: "2026-11-24T10:00:00Z".into(),
            finished_at: Some("2026-11-24T10:01:00Z".into()),
        },
    );

    assert_eq!(view.progress, 100);
    assert_eq!(view.state, SyncState::Ok);
    assert_eq!(view.requests, 12);
    assert_eq!(view.last_run.unwrap().id, "1");
}

#[test]
fn a_failed_run_keeps_the_failed_state() {
    let mut view = SyncView::default();

    apply_finished(
        &mut view,
        SyncRun {
            id: "2".into(),
            status: SyncStatusKind::Failed,
            requests: 0,
            started_at: "2026-11-24T10:00:00Z".into(),
            finished_at: None,
        },
    );

    assert_eq!(view.state, SyncState::Failed);
    assert_eq!(view.progress, 100);
}

#[test]
fn a_partial_run_keeps_the_partial_state() {
    let mut view = SyncView::default();

    apply_finished(
        &mut view,
        SyncRun {
            id: "3".into(),
            status: SyncStatusKind::Partial,
            requests: 4,
            started_at: "2026-11-24T10:00:00Z".into(),
            finished_at: Some("2026-11-24T10:01:00Z".into()),
        },
    );

    assert_eq!(view.state, SyncState::Partial);
}
