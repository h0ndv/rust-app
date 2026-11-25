pub mod linking;
pub mod models;
pub mod repo;

use std::path::Path;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::error::AppError;

const IN_MEMORY: &str = ":memory:";

/// Opens the pool. `:memory:` is limited to one connection: each additional
/// connection would see a different database.
pub async fn connect(path: &Path) -> Result<SqlitePool, AppError> {
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
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Internal(format!("could not create {}: {e}", parent.display()))
            })?;
        }
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
        .map_err(|e| AppError::Internal(format!("could not open the database: {e}")))
}

pub async fn migrate(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("migration failed: {e}")))
}
