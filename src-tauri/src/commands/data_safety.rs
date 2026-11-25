use tauri::State;

use crate::commands::{Cmd, RecoveryDto};
use crate::state::AppState;

#[tauri::command]
pub async fn export_data(state: State<'_, AppState>) -> Cmd<String> {
    let folder = state.repo.downloads_dir(&state.downloads_fallback).await?;
    let path = crate::export::write_snapshot(&folder, &state.pool).await?;

    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn clear_downloads(state: State<'_, AppState>) -> Cmd<u64> {
    let rows = state.repo.downloaded_paths().await?;
    let (removed, cleared) = clear_recorded_paths(&rows).await;

    for id in cleared {
        state.repo.clear_downloaded(&id).await?;
    }

    Ok(removed)
}

#[tauri::command]
pub async fn recovery_notice(
    recovery: State<'_, Option<crate::db::Recovery>>,
) -> Cmd<Option<RecoveryDto>> {
    Ok(recovery.as_ref().map(|item| RecoveryDto {
        backup_path: item.backup_path.clone(),
        reason: item.reason.clone(),
    }))
}

pub async fn clear_recorded_paths(rows: &[(String, String)]) -> (u64, Vec<String>) {
    let mut removed = 0;
    let mut cleared = Vec::new();

    for (id, path) in rows {
        match tokio::fs::symlink_metadata(path).await {
            Ok(metadata) if metadata.file_type().is_file() => {
                match tokio::fs::remove_file(path).await {
                    Ok(()) => {
                        removed += 1;
                        cleared.push(id.clone());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        cleared.push(id.clone());
                    }
                    Err(_) => {}
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                cleared.push(id.clone());
            }
            Err(_) => {}
        }
    }

    (removed, cleared)
}
