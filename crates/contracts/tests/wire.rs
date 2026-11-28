use contracts::*;

#[test]
fn session_state_is_tagged_lowercase() {
    let value = serde_json::to_value(SessionState::Expired(Source::Moodle)).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"state": "expired", "source": "moodle"})
    );
}

#[test]
fn filters_are_lowercase() {
    assert_eq!(serde_json::to_value(TaskFilter::NoData).unwrap(), "no_data");
    assert_eq!(
        serde_json::to_value(ResourceFilter::Downloaded).unwrap(),
        "downloaded"
    );
}

#[test]
fn error_payload_has_kind_and_message() {
    let value = serde_json::to_value(AppErrorPayload {
        kind: ErrorKind::Unauthorized,
        message: "Session expired".into(),
    })
    .unwrap();
    assert_eq!(value["kind"], "unauthorized");
}

#[test]
fn session_state_round_trips() {
    let states = [
        SessionState::Disconnected,
        SessionState::Connecting(Source::Ellucian),
        SessionState::Connected,
        SessionState::Expired(Source::Banner),
        SessionState::Partial,
    ];

    for state in states {
        let value = serde_json::to_value(&state).unwrap();
        let back: SessionState = serde_json::from_value(value).unwrap();
        assert_eq!(back, state);
    }
}

#[test]
fn multiword_error_kinds_keep_their_snake_case() {
    assert_eq!(
        serde_json::to_value(ErrorKind::NotFound).unwrap(),
        "not_found"
    );
    assert_eq!(
        serde_json::to_value(ErrorKind::RateLimited).unwrap(),
        "rate_limited"
    );
}

#[test]
fn course_detail_ignores_fields_the_interface_does_not_read() {
    let value = serde_json::json!({
        "group": { "id": "g1", "code": "IGR-1", "name": "Ingeniería", "sections": [] },
        "tasks": [{
            "id": "assign:1", "course_id": "moodle:1", "section_id": null, "cmid": 1,
            "name": "Tarea", "modtype": "assign", "url": null, "due_at": 10,
            "open_at": null, "cutoff_at": null, "status": "pending", "submitted": null,
            "graded": null, "grade": null, "max_grade": null, "description": "",
            "content_hash": null, "first_seen_at": null, "acknowledged_at": null,
            "archived_at": null, "last_seen_at": null
        }],
        "resources": [],
        "grades": [],
        "schedule": []
    });

    let detail: CourseDetailDto = serde_json::from_value(value).unwrap();
    assert_eq!(detail.tasks[0].name, "Tarea");
}
