use std::sync::Mutex;
use std::time::Duration;

use serde::de::DeserializeOwned;
use tauri::WebviewWindow;

use crate::error::AppError;

/// Runs JavaScript in the Banner window and waits for the JSON returned by
/// `eval_with_callback`. With no answer in 30 seconds, it is cut off.
pub async fn eval_json<T: DeserializeOwned>(
    window: &WebviewWindow,
    js: &str,
) -> Result<T, AppError> {
    let (sender, receiver) = tokio::sync::oneshot::channel::<String>();
    let sender = Mutex::new(Some(sender));

    window
        .eval_with_callback(js.to_string(), move |result| {
            if let Ok(mut guard) = sender.lock()
                && let Some(sender) = guard.take()
            {
                let _ = sender.send(result);
            }
        })
        .map_err(|e| AppError::Internal(format!("could not evaluate in Banner: {e}")))?;

    let text = tokio::time::timeout(Duration::from_secs(30), receiver)
        .await
        .map_err(|_| AppError::Network("Banner did not respond within 30 seconds".into()))?
        .map_err(|_| AppError::Network("the Banner window was closed".into()))?;

    serde_json::from_str(&text)
        .map_err(|e| AppError::Parse(format!("invalid Banner response: {e}")))
}
