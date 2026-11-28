use tauri_plugin_notification::NotificationExt;

use crate::commands::Cmd;
use crate::error::AppError;

#[tauri::command]
pub async fn notification_permission(app: tauri::AppHandle) -> Cmd<bool> {
    use tauri_plugin_notification::PermissionState;

    app.notification()
        .permission_state()
        .map(|state| state == PermissionState::Granted)
        .map_err(|error| AppError::Internal(error.to_string()))
}

#[tauri::command]
pub async fn request_notification_permission(app: tauri::AppHandle) -> Cmd<bool> {
    use tauri_plugin_notification::PermissionState;

    app.notification()
        .request_permission()
        .map(|state| state == PermissionState::Granted)
        .map_err(|error| AppError::Internal(error.to_string()))
}

#[tauri::command]
pub async fn send_notification(app: tauri::AppHandle, title: String, body: String) -> Cmd<()> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|error| AppError::Internal(error.to_string()))
}
