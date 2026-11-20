use rust_app_lib::commands::dto::{DashboardDto, GradesDto, ResourceFilter, TaskFilter};

#[test]
fn dashboard_serializes_expected_fields() {
    let dto = DashboardDto::empty();
    let json = serde_json::to_value(&dto).unwrap();

    assert!(json.get("overdue").is_some());
    assert!(json.get("due_soon").is_some());
    assert!(json.get("new_resources").is_some());
    assert!(json.get("next_classes").is_some());
}

#[test]
fn filters_arrive_lowercase() {
    let task: TaskFilter = serde_json::from_str("\"overdue\"").unwrap();
    assert_eq!(task, TaskFilter::Overdue);

    let resource: ResourceFilter = serde_json::from_str("\"new\"").unwrap();
    assert_eq!(resource, ResourceFilter::New);
}

#[test]
fn grades_separate_banner_from_moodle() {
    let dto = GradesDto::default();
    let json = serde_json::to_value(&dto).unwrap();

    assert!(json.get("banner").is_some());
    assert!(json.get("moodle_totals").is_some());
    assert!(json.get("terms").is_some());
    assert!(json.get("gpa").is_some());
}

#[test]
fn settings_have_default_values() {
    let dto = rust_app_lib::commands::dto::SettingsDto::default();
    assert_eq!(dto.sync_interval_minutes, 30);
    assert_eq!(dto.theme, "system");
    assert!(dto.notifications);
}
