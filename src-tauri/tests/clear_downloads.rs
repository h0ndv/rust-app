use rust_app_lib::db::repo::Repo;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rust-app-{name}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn removes_only_recorded_files_and_missing_paths_clear() {
    let dir = temp_dir("clear");
    let file = dir.join("guia.pdf");
    std::fs::write(&file, b"pdf").unwrap();
    let unlisted = dir.join("unlisted.pdf");
    std::fs::write(&unlisted, b"keep").unwrap();
    let folder = dir.join("folder");
    std::fs::create_dir_all(&folder).unwrap();

    let rows = vec![
        ("f1".to_string(), file.display().to_string()),
        (
            "f2".to_string(),
            dir.join("missing.pdf").display().to_string(),
        ),
        ("f3".to_string(), folder.display().to_string()),
    ];
    let (removed, cleared) = rust_app_lib::commands::data_safety::clear_recorded_paths(&rows).await;

    assert_eq!(removed, 1);
    assert_eq!(cleared, vec!["f1".to_string(), "f2".to_string()]);
    assert!(!file.exists());
    assert!(folder.exists());
    assert!(unlisted.exists());

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        let locked = dir.join("locked.pdf");
        std::fs::write(&locked, b"x").unwrap();
        let handle = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&locked)
            .unwrap();

        let (locked_removals, locked_cleared) =
            rust_app_lib::commands::data_safety::clear_recorded_paths(&[(
                "f4".to_string(),
                locked.display().to_string(),
            )])
            .await;
        assert_eq!(locked_removals, 0);
        assert!(locked_cleared.is_empty());

        drop(handle);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let target = dir.join("target.pdf");
        std::fs::write(&target, b"target").unwrap();
        let link = dir.join("link.pdf");
        symlink(&target, &link).unwrap();

        let (_, linked) = rust_app_lib::commands::data_safety::clear_recorded_paths(&[(
            "f5".to_string(),
            link.display().to_string(),
        )])
        .await;
        assert!(linked.is_empty());
        assert!(link.exists());
    }

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn clears_the_database_rows_for_removed_and_missing_files() {
    let pool = rust_app_lib::db::connect(std::path::Path::new(":memory:"))
        .await
        .unwrap();
    rust_app_lib::db::migrate(&pool).await.unwrap();
    let repo = Repo::new(pool.clone());

    let dir = temp_dir("rows");
    let file = dir.join("guia.pdf");
    std::fs::write(&file, b"pdf").unwrap();
    let folder = dir.join("folder");
    std::fs::create_dir_all(&folder).unwrap();

    for (id, path) in [
        ("f1", file.display().to_string()),
        ("f2", dir.join("missing.pdf").display().to_string()),
        ("f3", folder.display().to_string()),
    ] {
        sqlx::query(
            "INSERT INTO files (id, name, url, local_path)
             VALUES (?, 'name', 'https://example.com', ?)",
        )
        .bind(id)
        .bind(path)
        .execute(&pool)
        .await
        .unwrap();
    }

    let rows = repo.downloaded_paths().await.unwrap();
    let (removed, cleared) = rust_app_lib::commands::data_safety::clear_recorded_paths(&rows).await;
    for id in &cleared {
        repo.clear_downloaded(id).await.unwrap();
    }

    assert_eq!(removed, 1);
    assert_eq!(cleared, vec!["f1".to_string(), "f2".to_string()]);

    let remaining: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT id, local_path FROM files ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        remaining,
        vec![
            ("f1".to_string(), None),
            ("f2".to_string(), None),
            ("f3".to_string(), Some(folder.display().to_string())),
        ]
    );

    std::fs::remove_dir_all(&dir).ok();
}
