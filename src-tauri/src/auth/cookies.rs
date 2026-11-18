use tauri::WebviewWindow;
use url::Url;

use crate::error::AppError;

/// Reads the WebView2 profile cookies for `url` and leaves them as a header.
/// `cookies_for_url` blocks if it runs on the main thread: it always goes in
/// `spawn_blocking`.
pub async fn header_for(window: &WebviewWindow, url: &str) -> Result<String, AppError> {
    let parsed =
        Url::parse(url).map_err(|e| AppError::Internal(format!("URL de cookies inválida: {e}")))?;
    let window = window.clone();

    let cookies = tokio::task::spawn_blocking(move || window.cookies_for_url(parsed))
        .await
        .map_err(|e| AppError::Internal(format!("tarea de cookies cancelada: {e}")))?
        .map_err(|e| AppError::Internal(format!("no se pudieron leer las cookies: {e}")))?;

    Ok(cookies
        .iter()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<_>>()
        .join("; "))
}
