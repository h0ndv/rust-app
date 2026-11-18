use std::sync::Arc;

use sqlx::SqlitePool;

use crate::db::repo::Repo;
use crate::sync::SyncOrchestrator;

/// Shared app state. `SessionManager` and the downloads are handled
/// as separate resources, so the tests do not need an `AppHandle`.
pub struct AppState {
    pub pool: SqlitePool,
    pub repo: Repo,
    pub sync: Arc<SyncOrchestrator>,
}

impl AppState {
    pub fn new(pool: SqlitePool, repo: Repo, sync: Arc<SyncOrchestrator>) -> Self {
        Self { pool, repo, sync }
    }
}
