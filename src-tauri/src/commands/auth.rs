use std::sync::Arc;

use tauri::State;

use crate::auth::{SessionManager, SessionState, Source};

#[tauri::command]
pub async fn connect(
    session: State<'_, Arc<SessionManager>>,
    source: Option<Source>,
) -> Result<SessionState, String> {
    match source {
        Some(source) => {
            session
                .ensure(source)
                .await
                .map_err(|error| error.to_string())?;
            Ok(session.state().await)
        }
        None => session
            .connect_all()
            .await
            .map_err(|error| error.to_string()),
    }
}

#[tauri::command]
pub async fn session_state(
    session: State<'_, Arc<SessionManager>>,
) -> Result<SessionState, String> {
    Ok(session.state().await)
}
