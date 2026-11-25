use sqlx::SqlitePool;

async fn seeded_pool() -> SqlitePool {
    let pool = rust_app_lib::db::connect(std::path::Path::new(":memory:"))
        .await
        .unwrap();
    rust_app_lib::db::migrate(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO course_groups (id, code, name)
         VALUES ('group:1', 'IGR-001', 'Mineria de Datos')",
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
    sqlx::query(
        "INSERT INTO activities (id, course_id, name, modtype)
         VALUES ('activity:1', 'course:1', 'Lab 1', 'assign')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO files (id, course_id, name, url)
         VALUES ('file:1', 'course:1', 'Guia lab.pdf', 'https://example.com/guia.pdf')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO announcements (id, course_id, title)
         VALUES ('announcement:1', 'course:1', 'Aviso lab')",
    )
    .execute(&pool)
    .await
    .unwrap();

    for (kind, id, title) in [
        ("activity", "activity:1", "Lab 1"),
        ("file", "file:1", "Guia lab.pdf"),
        ("announcement", "announcement:1", "Aviso lab"),
    ] {
        sqlx::query(
            "INSERT INTO search_index (kind, entity_id, title, body)
             VALUES (?, ?, ?, '')",
        )
        .bind(kind)
        .bind(id)
        .bind(title)
        .execute(&pool)
        .await
        .unwrap();
    }

    pool
}

#[tokio::test]
async fn search_hits_resolve_the_course_group() {
    let pool = seeded_pool().await;

    let hits = rust_app_lib::commands::data::search_hits(&pool, "lab")
        .await
        .unwrap();

    let activity = hits.iter().find(|hit| hit.kind == "activity").unwrap();
    let file = hits.iter().find(|hit| hit.kind == "file").unwrap();
    let announcement = hits.iter().find(|hit| hit.kind == "announcement").unwrap();
    assert_eq!(activity.group_id.as_deref(), Some("group:1"));
    assert_eq!(file.group_id.as_deref(), Some("group:1"));
    assert_eq!(announcement.group_id.as_deref(), Some("group:1"));
}

#[tokio::test]
async fn orphan_index_rows_have_no_group() {
    let pool = seeded_pool().await;
    sqlx::query(
        "INSERT INTO search_index (kind, entity_id, title, body)
         VALUES ('activity', 'missing', 'huerfano lab', '')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let hits = rust_app_lib::commands::data::search_hits(&pool, "huerfano")
        .await
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].group_id, None);
}

#[tokio::test]
async fn news_items_resolve_the_course_group() {
    let pool = seeded_pool().await;

    let items = rust_app_lib::commands::data::news_items(&pool)
        .await
        .unwrap();

    assert_eq!(items.len(), 3);
    assert!(
        items
            .iter()
            .all(|item| item.group_id.as_deref() == Some("group:1"))
    );
}
