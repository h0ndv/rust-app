use std::sync::Arc;

use tauri::State;

use crate::auth::{SessionManager, SessionState, Source};
use crate::error::AppError;

#[tauri::command]
pub async fn connect(
    session: State<'_, Arc<SessionManager>>,
    source: Option<Source>,
) -> Result<SessionState, AppError> {
    match source {
        Some(source) => {
            session.ensure(source).await?;
            Ok(session.state().await)
        }
        None => session.connect_all().await,
    }
}

#[tauri::command]
pub async fn session_state(
    session: State<'_, Arc<SessionManager>>,
) -> Result<SessionState, AppError> {
    Ok(session.state().await)
}
