use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::db::repo::Repo;
use crate::sync::SyncOrchestrator;

pub struct AppState {
    pub pool: SqlitePool,
    pub repo: Repo,
    pub sync: Arc<SyncOrchestrator>,
    pub downloads_fallback: PathBuf,
}

impl AppState {
    pub fn new(
        pool: SqlitePool,
        repo: Repo,
        sync: Arc<SyncOrchestrator>,
        downloads_fallback: PathBuf,
    ) -> Self {
        Self {
            pool,
            repo,
            sync,
            downloads_fallback,
        }
    }
}
