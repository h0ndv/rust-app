#[tokio::test]
async fn moves_a_corrupt_database_aside_and_starts_fresh() {
    let dir = std::env::temp_dir().join(format!("rust-app-corrupt-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("panel.db");
    let original = b"this is not a database";
    std::fs::write(&path, original).unwrap();

    let (pool, recovery) = rust_app_lib::db::open_with_recovery(&path).await.unwrap();

    let recovery = recovery.expect("corrupt database must be reported");
    let backup = std::path::Path::new(&recovery.backup_path);
    assert_eq!(std::fs::read(backup).unwrap(), original);

    let tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master
         WHERE type = 'table' AND name IN ('_sqlx_migrations', 'files')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tables, 2);

    pool.close().await;
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn a_valid_database_is_not_moved() {
    let dir = std::env::temp_dir().join(format!("rust-app-valid-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("panel.db");

    let (first, recovery) = rust_app_lib::db::open_with_recovery(&path).await.unwrap();
    assert!(recovery.is_none());
    first.close().await;

    let (second, recovery) = rust_app_lib::db::open_with_recovery(&path).await.unwrap();
    assert!(recovery.is_none());
    second.close().await;

    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(windows)]
#[tokio::test]
async fn a_locked_database_is_not_moved() {
    use std::os::windows::fs::OpenOptionsExt;

    let dir = std::env::temp_dir().join(format!("rust-app-locked-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("panel.db");

    let (pool, recovery) = rust_app_lib::db::open_with_recovery(&path).await.unwrap();
    assert!(recovery.is_none());
    pool.close().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();

    let result = rust_app_lib::db::open_with_recovery(&path).await;
    assert!(result.is_err());

    let backups = std::fs::read_dir(&dir)
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".corrupt-")
        })
        .count();
    assert_eq!(backups, 0);

    drop(lock);
    std::fs::remove_dir_all(&dir).ok();
}
