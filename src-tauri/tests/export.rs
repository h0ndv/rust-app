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
    assert!(json.get("search_index").is_none());
}
