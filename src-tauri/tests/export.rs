#[tokio::test]
async fn snapshot_contains_the_durable_tables() {
    let pool = rust_app_lib::db::connect(std::path::Path::new(":memory:"))
        .await
        .unwrap();
    rust_app_lib::db::migrate(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO course_groups (id, code, name)
         VALUES ('group:1', 'IGR-001', 'Mineria')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO courses (id, group_id, source, fullname)
         VALUES ('course:1', 'group:1', 'moodle', 'Mineria de Datos')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO settings (key, value) VALUES ('theme', 'dark')")
        .execute(&pool)
        .await
        .unwrap();

    let snapshot = rust_app_lib::export::snapshot(&pool).await.unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&rust_app_lib::export::snapshot_json(&snapshot).unwrap()).unwrap();

    assert_eq!(json["version"], 1);
    assert_eq!(json["course_groups"][0]["name"], "Mineria");
    assert_eq!(json["courses"][0]["fullname"], "Mineria de Datos");
    assert_eq!(json["settings"][0]["key"], "theme");
    for key in [
        "course_groups",
        "courses",
        "sections",
        "activities",
        "files",
        "announcements",
        "grade_items",
        "grade_totals",
        "schedule_events",
        "settings",
        "source_status",
    ] {
        assert!(json.get(key).is_some(), "missing {key}");
    }
    assert!(json.get("search_index").is_none());
}

#[tokio::test]
async fn writes_a_readable_json_file() {
    let pool = rust_app_lib::db::connect(std::path::Path::new(":memory:"))
        .await
        .unwrap();
    rust_app_lib::db::migrate(&pool).await.unwrap();
    let dir = std::env::temp_dir().join(format!("rust-app-export-{}", uuid::Uuid::new_v4()));

    let path = rust_app_lib::export::write_snapshot(&dir, &pool)
        .await
        .unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert_eq!(json["version"], 1);
    assert!(path.starts_with(&dir));

    std::fs::remove_dir_all(&dir).ok();
}
