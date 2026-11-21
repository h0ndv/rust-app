use rust_app_lib::commands::dto::{DashboardDto, GradesDto, NewsKind, ResourceFilter, TaskFilter};

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

    let no_data: TaskFilter = serde_json::from_str("\"no_data\"").unwrap();
    assert_eq!(no_data, TaskFilter::NoData);

    let resource: ResourceFilter = serde_json::from_str("\"new\"").unwrap();
    assert_eq!(resource, ResourceFilter::New);
}

#[test]
fn news_kinds_are_typed() {
    let kind: NewsKind = serde_json::from_str("\"activity\"").unwrap();
    assert_eq!(kind, NewsKind::Activity);
    assert_eq!(kind.as_str(), "activity");

    assert!(serde_json::from_str::<NewsKind>("\"bogus\"").is_err());
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
    assert_eq!(dto.language, "en");
}
