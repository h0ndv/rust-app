use std::time::Duration;

use tauri::WebviewWindow;
use url::Url;

use crate::error::AppError;

const ATTEMPTS: u8 = 2;

/// Reads the WebView2 profile cookies for `url` and leaves them as a header.
/// `cookies_for_url` blocks if it runs on the main thread: it always goes in
/// `spawn_blocking`. A webview that is mid-navigation can drop the reply
/// channel, so the read retries once before giving up.
pub async fn header_for(window: &WebviewWindow, url: &str) -> Result<String, AppError> {
    let parsed =
        Url::parse(url).map_err(|e| AppError::Internal(format!("invalid cookie URL: {e}")))?;

    let mut attempt = 1;
    loop {
        let window = window.clone();
        let parsed = parsed.clone();
        let result = tokio::task::spawn_blocking(move || window.cookies_for_url(parsed)).await;

        match result {
            Ok(Ok(cookies)) => {
                return Ok(cookies
                    .iter()
                    .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
                    .collect::<Vec<_>>()
                    .join("; "));
            }
            Ok(Err(error)) if attempt < ATTEMPTS => {
                tracing::warn!(%error, attempt, "cookie read failed, retrying");
            }
            Ok(Err(error)) => {
                return Err(AppError::Internal(format!(
                    "could not read cookies: {error}"
                )));
            }
            Err(error) if attempt < ATTEMPTS => {
                tracing::warn!(%error, attempt, "cookie read task cancelled, retrying");
            }
            Err(error) => {
                return Err(AppError::Internal(format!(
                    "cookie task cancelled: {error}"
                )));
            }
        }

        attempt += 1;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}
