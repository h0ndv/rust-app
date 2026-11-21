use tauri::State;

use crate::state::AppState;
use crate::sync::{SyncStatus, SyncTrigger};

#[tauri::command]
pub fn start_sync(state: State<'_, AppState>) -> SyncStatus {
    state.sync.start(SyncTrigger::Manual)
}

#[tauri::command]
pub fn cancel_sync(state: State<'_, AppState>) {
    state.sync.cancel();
}

#[tauri::command]
pub fn sync_status(state: State<'_, AppState>) -> SyncStatus {
    state.sync.status()
}
