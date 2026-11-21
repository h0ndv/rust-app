use std::path::Path;

use rust_app_lib::db::models::{Activity, Course};
use rust_app_lib::db::repo::Repo;
use rust_app_lib::db::{connect, migrate};

async fn test_pool() -> sqlx::SqlitePool {
    let pool = connect(Path::new(":memory:")).await.unwrap();
    migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn sync_interval_defaults_and_clamps() {
    let pool = test_pool().await;
    let repo = Repo::new(pool.clone());

    assert_eq!(repo.sync_interval_minutes().await.unwrap(), 30);

    for (value, expected) in [("5", 5), ("0", 5), ("nope", 30), ("99999", 1440)] {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ('sync_interval_minutes', ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(value)
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(repo.sync_interval_minutes().await.unwrap(), expected);
    }
}

#[tokio::test]
async fn downloads_dir_uses_setting_or_fallback() {
    let pool = test_pool().await;
    let repo = Repo::new(pool.clone());
    let fallback = std::path::Path::new("C:\\fallback");

    assert_eq!(
        repo.downloads_dir(fallback).await.unwrap(),
        fallback.to_path_buf()
    );

    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('downloads_dir', 'C:\\chosen')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        repo.downloads_dir(fallback).await.unwrap(),
        std::path::PathBuf::from("C:\\chosen")
    );

    sqlx::query("UPDATE settings SET value = '   ' WHERE key = 'downloads_dir'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        repo.downloads_dir(fallback).await.unwrap(),
        fallback.to_path_buf()
    );
}

#[tokio::test]
async fn creates_all_tables() {
    let pool = test_pool().await;
    for table in [
        "course_groups",
        "courses",
        "sections",
        "activities",
        "files",
        "grade_items",
        "grade_totals",
        "schedule_events",
        "announcements",
        "sync_runs",
        "source_status",
        "settings",
        "events",
        "course_links",
        "search_index",
    ] {
        let exists: i64 =
            sqlx::query_scalar("SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?")
                .bind(table)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(exists, 1, "missing {table}");
    }
}

#[tokio::test]
async fn schema_can_be_applied_twice() {
    let pool = test_pool().await;
    migrate(&pool).await.unwrap();
}

#[tokio::test]
async fn courses_and_activities_round_trip() {
    let pool = test_pool().await;
    let repo = Repo::new(pool.clone());

    let course = Course {
        id: "moodle:821065".into(),
        group_id: Some("IGR-00167".into()),
        source: "moodle".into(),
        moodle_id: Some(821065),
        banner_crn: None,
        shortname: Some("30061.202620".into()),
        fullname: "Minería de Datos (IGR-00167-00G)".into(),
        section: Some("00G".into()),
        term: Some("202620".into()),
        teacher: Some("Santo Tomás".into()),
        moodle_url: Some("https://aulasvirtuales.santotomas.cl/course/view.php?id=821065".into()),
        banner_url: None,
        last_synced_at: Some("2026-11-17T10:00:00Z".into()),
    };

    repo.upsert_course(&course).await.unwrap();
    assert_eq!(repo.course_count().await.unwrap(), 1);

    let fetched: Course = sqlx::query_as("SELECT * FROM courses WHERE id = ?")
        .bind(&course.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fetched.fullname, course.fullname);
    assert_eq!(fetched.moodle_id, Some(821065));

    sqlx::query(
        "INSERT INTO activities (id, course_id, cmid, name, modtype, due_at, submitted)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("assign:22053263")
    .bind(&course.id)
    .bind(22053263_i64)
    .bind("Taller de Apriori")
    .bind("assign")
    .bind(1_800_000_000_i64)
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

    let activity: Activity = sqlx::query_as("SELECT * FROM activities WHERE id = ?")
        .bind("assign:22053263")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(activity.name, "Taller de Apriori");
    assert_eq!(activity.submitted, Some(false));
    assert_eq!(activity.description, "");
    assert!(activity.acknowledged_at.is_none());
}

/// Characterization: the real startup uses a file in `app_data_dir`,
/// not `:memory:`. It covers the path with `create_if_missing` and WAL.
#[tokio::test]
async fn file_database_persists_between_opens() {
    let dir = std::env::temp_dir().join(format!("rust-app-test-{}", uuid::Uuid::new_v4()));
    let db_path = dir.join("panel.db");

    {
        let pool = connect(&db_path).await.unwrap();
        migrate(&pool).await.unwrap();
        Repo::new(pool.clone())
            .upsert_course(&Course {
                id: "banner:12345".into(),
                group_id: None,
                source: "banner".into(),
                moodle_id: None,
                banner_crn: Some("12345".into()),
                shortname: None,
                fullname: "Curso Sin Aula Virtual".into(),
                section: Some("00G".into()),
                term: Some("202620".into()),
                teacher: None,
                moodle_url: None,
                banner_url: None,
                last_synced_at: None,
            })
            .await
            .unwrap();
        pool.close().await;
    }

    let pool = connect(&db_path).await.unwrap();
    migrate(&pool).await.unwrap();
    assert_eq!(Repo::new(pool.clone()).course_count().await.unwrap(), 1);
    pool.close().await;

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn search_index_answers_match() {
    let pool = test_pool().await;

    sqlx::query("INSERT INTO search_index (kind, entity_id, title, body) VALUES (?, ?, ?, ?)")
        .bind("activity")
        .bind("assign:22053263")
        .bind("Taller de Apriori")
        .bind("reglas de asociación sobre boletas")
        .execute(&pool)
        .await
        .unwrap();

    let found: i64 =
        sqlx::query_scalar("SELECT count(*) FROM search_index WHERE search_index MATCH ?")
            .bind("apriori")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(found, 1);
}

#[tokio::test]
async fn file_database_uses_wal_normal_synchronous_and_size_limit() {
    let dir = std::env::temp_dir().join(format!("rust-app-pragmas-{}", uuid::Uuid::new_v4()));
    let db_path = dir.join("panel.db");

    let pool = connect(&db_path).await.unwrap();
    migrate(&pool).await.unwrap();

    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();
    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(&pool)
        .await
        .unwrap();
    let limit: i64 = sqlx::query_scalar("PRAGMA journal_size_limit")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(journal, "wal");
    assert_eq!(synchronous, 1, "NORMAL");
    assert_eq!(limit, 8 * 1024 * 1024);

    pool.close().await;
    std::fs::remove_dir_all(&dir).ok();
}
