use tauri::State;

use crate::state::AppState;
use crate::sync::{SyncRun, SyncStatus, SyncTrigger};

#[tauri::command]
pub async fn start_sync(state: State<'_, AppState>) -> Result<SyncRun, String> {
    state
        .sync
        .run(SyncTrigger::Manual)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cancel_sync(state: State<'_, AppState>) {
    state.sync.cancel();
}

#[tauri::command]
pub fn sync_status(state: State<'_, AppState>) -> SyncStatus {
    state.sync.status()
}
