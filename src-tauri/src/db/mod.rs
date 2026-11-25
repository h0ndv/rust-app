pub mod linking;
pub mod models;
pub mod repo;

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::error::AppError;

const IN_MEMORY: &str = ":memory:";

#[derive(Debug, Clone, Serialize)]
pub struct Recovery {
    pub backup_path: String,
    pub reason: String,
}

/// Opens the database. Only a file that fails SQLite's integrity check is
/// moved aside; a locked, missing or migration-broken database returns its
/// error so a valid file is never renamed.
pub async fn open_with_recovery(path: &Path) -> Result<(SqlitePool, Option<Recovery>), AppError> {
    prepare_parent(path)?;

    let pool = match open_pool(path).await {
        Ok(pool) => pool,
        Err(error) if is_corruption(&error) => {
            return recover(path, open_error(error).to_string()).await;
        }
        Err(error) => return Err(open_error(error)),
    };

    match quick_check(&pool).await {
        Ok(true) => {}
        Ok(false) => {
            pool.close().await;
            return recover(path, "quick_check reported damage".to_string()).await;
        }
        Err(error) if is_corruption(&error) => {
            pool.close().await;
            return recover(path, open_error(error).to_string()).await;
        }
        Err(error) => {
            pool.close().await;
            return Err(open_error(error));
        }
    }

    match migrate(&pool).await {
        Ok(()) => Ok((pool, None)),
        Err(error) => {
            pool.close().await;
            Err(error)
        }
    }
}

async fn recover(path: &Path, reason: String) -> Result<(SqlitePool, Option<Recovery>), AppError> {
    if path.as_os_str() == IN_MEMORY {
        return Err(AppError::Internal(reason));
    }

    let is_file = std::fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file())
        .unwrap_or(false);
    if !is_file {
        return Err(AppError::Internal(format!(
            "{reason}; the database path is not a file"
        )));
    }

    let backup = backup_path(path);
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();

    for suffix in ["-wal", "-shm"] {
        let source = sidecar_path(path, suffix);
        if !source.exists() {
            continue;
        }
        let target = sidecar_path(&backup, suffix);
        match rename_retry(&source, &target).await {
            Ok(()) => moved.push((target, source)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                rollback(&moved);
                return Err(AppError::Internal(format!(
                    "{reason}; could not move {}: {error}",
                    source.display()
                )));
            }
        }
    }

    if let Err(error) = rename_retry(path, &backup).await {
        rollback(&moved);
        return Err(AppError::Internal(format!(
            "{reason}; could not move the database aside: {error}"
        )));
    }

    let pool = open_pool(path).await.map_err(open_error)?;
    migrate(&pool).await?;

    Ok((
        pool,
        Some(Recovery {
            backup_path: backup.display().to_string(),
            reason,
        }),
    ))
}

fn rollback(moved: &[(PathBuf, PathBuf)]) {
    for (target, source) in moved.iter().rev() {
        let _ = std::fs::rename(target, source);
    }
}

async fn rename_retry(source: &Path, target: &Path) -> std::io::Result<()> {
    let mut attempts = 0;

    loop {
        match std::fs::rename(source, target) {
            Ok(()) => return Ok(()),
            Err(error)
                if cfg!(windows)
                    && attempts < 10
                    && matches!(error.raw_os_error(), Some(5) | Some(32)) =>
            {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => return Err(error),
        }
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("panel.db");

    path.with_file_name(format!(
        "{name}.corrupt-{}",
        Utc::now().format("%Y%m%d-%H%M%S-%3f")
    ))
}

fn sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", path.display()))
}

fn is_corruption(error: &sqlx::Error) -> bool {
    let sqlx::Error::Database(database) = error else {
        return false;
    };
    let Some(code) = database.code() else {
        return false;
    };

    matches!(code.parse::<i32>().map(|code| code & 0xff), Ok(11 | 26))
}

async fn quick_check(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let rows: Vec<String> = sqlx::query_scalar("PRAGMA quick_check")
        .fetch_all(pool)
        .await?;

    Ok(rows.iter().all(|row| row.eq_ignore_ascii_case("ok")))
}

fn open_error(error: sqlx::Error) -> AppError {
    AppError::Internal(format!("could not open the database: {error}"))
}

fn prepare_parent(path: &Path) -> Result<(), AppError> {
    if path.as_os_str() == IN_MEMORY {
        return Ok(());
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Internal(format!("could not create {}: {e}", parent.display()))
        })?;
    }

    Ok(())
}

/// Opens the pool. `:memory:` is limited to one connection: each additional
/// connection would see a different database.
pub async fn connect(path: &Path) -> Result<SqlitePool, AppError> {
    prepare_parent(path)?;
    open_pool(path).await.map_err(open_error)
}

async fn open_pool(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let in_memory = path.as_os_str() == IN_MEMORY;

    let mut options = SqliteConnectOptions::new()
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(10))
        // WAL plus NORMAL is durable enough for a local cache and avoids one
        // fsync per commit. The WAL is capped so it does not grow unbounded.
        .synchronous(SqliteSynchronous::Normal)
        .pragma("journal_size_limit", "8388608");

    let max_connections = if in_memory {
        options = options.in_memory(true);
        1
    } else {
        options = options
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);
        8
    };

    SqlitePoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect_with(options)
        .await
}

pub async fn migrate(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("migration failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn moves_a_corrupt_file_and_its_sidecars() {
        let dir = std::env::temp_dir().join(format!("rust-app-sidecars-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("panel.db");
        std::fs::write(&path, b"bad database").unwrap();
        std::fs::write(sidecar_path(&path, "-wal"), b"wal bytes").unwrap();
        std::fs::write(sidecar_path(&path, "-shm"), b"shm bytes").unwrap();

        let (pool, recovery) = recover(&path, "test".to_string()).await.unwrap();
        let backup = recovery.expect("recovery must be reported").backup_path;
        let backup = Path::new(&backup);

        assert_eq!(std::fs::read(backup).unwrap(), b"bad database");
        assert_eq!(
            std::fs::read(sidecar_path(backup, "-wal")).unwrap(),
            b"wal bytes"
        );
        assert_eq!(
            std::fs::read(sidecar_path(backup, "-shm")).unwrap(),
            b"shm bytes"
        );

        pool.close().await;
        std::fs::remove_dir_all(&dir).ok();
    }
}
