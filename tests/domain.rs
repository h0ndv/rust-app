#![cfg(not(target_arch = "wasm32"))]

use contracts::Source;
use rust_app_ui::domain::{
    ITEM_KIND_KEY, ItemKind, SOURCE_LABEL, SyncState, TASK_STATUS_KEY, TaskStatus, item_kind,
    source_label, task_status,
};

#[test]
fn legacy_status_and_kind_are_normalized() {
    assert_eq!(task_status(Some("entregada")), Some(TaskStatus::Submitted));
    assert_eq!(task_status(Some("unknown")), None);
    assert_eq!(item_kind(Some("archivo")), Some(ItemKind::File));
    assert_eq!(item_kind(Some("pdf")), Some(ItemKind::Other));
}

#[test]
fn canonical_values_pass_through() {
    assert_eq!(task_status(Some("overdue")), Some(TaskStatus::Overdue));
    assert_eq!(task_status(Some("no_data")), Some(TaskStatus::NoData));
    assert_eq!(task_status(None), None);
    assert_eq!(task_status(Some("")), None);

    assert_eq!(
        item_kind(Some("announcement")),
        Some(ItemKind::Announcement)
    );
    assert_eq!(item_kind(Some("file")), Some(ItemKind::File));
    assert_eq!(item_kind(None), None);
}

#[test]
fn keys_are_indexed_by_their_enum() {
    assert_eq!(
        TASK_STATUS_KEY[TaskStatus::Overdue as usize],
        "status.overdue"
    );
    assert_eq!(
        TASK_STATUS_KEY[TaskStatus::Submitted as usize],
        "status.submitted"
    );
    assert_eq!(ITEM_KIND_KEY[ItemKind::File as usize], "kinds.file");
    assert_eq!(ITEM_KIND_KEY[ItemKind::Other as usize], "kinds.other");
}

#[test]
fn sources_have_display_labels() {
    assert_eq!(SOURCE_LABEL[Source::Moodle as usize], "Moodle");
    assert_eq!(source_label(Source::Ellucian), "Ellucian");
    assert_eq!(source_label(Source::Banner), "Banner");
}

#[test]
fn sync_state_defaults_to_idle() {
    assert_eq!(SyncState::default(), SyncState::Idle);
}
