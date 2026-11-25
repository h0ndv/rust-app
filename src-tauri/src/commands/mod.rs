pub mod auth;
pub mod data;
pub mod dto;
pub mod sync;

pub use auth::*;
pub use data::*;
pub use dto::*;
pub use sync::*;

use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::error::AppError;
use crate::state::AppState;
use crate::sync::Downloads;

pub type Cmd<T> = Result<T, AppError>;

pub const BROWSER_WINDOW: &str = "browser";

#[tauri::command]
pub async fn open_in_app(app: AppHandle, url: String) -> Cmd<()> {
    let parsed = crate::security::parse_trusted(&url)?;

    let window = match app.get_webview_window(BROWSER_WINDOW) {
        Some(window) => window,
        None => {
            WebviewWindowBuilder::new(&app, BROWSER_WINDOW, WebviewUrl::External(parsed.clone()))
                .title("rust-app")
                .inner_size(1100.0, 800.0)
                .on_navigation(crate::security::trusted_source_url)
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                .build()
                .map_err(|error| AppError::Internal(error.to_string()))?
        }
    };

    window
        .navigate(parsed)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
pub fn download_file(downloads: State<'_, Downloads>, file_id: String) -> Cmd<()> {
    downloads.enqueue(file_id);
    Ok(())
}

const CLEAR_ORDER: [&str; 14] = [
    "events",
    "search_index",
    "announcements",
    "files",
    "activities",
    "sections",
    "grade_totals",
    "grade_items",
    "schedule_events",
    "course_links",
    "courses",
    "course_groups",
    "source_status",
    "sync_runs",
];

/// Deletes the local data. The WebView2 profile is left untouched.
#[tauri::command]
pub async fn clear_data(state: State<'_, AppState>) -> Cmd<u64> {
    let mut deleted = 0u64;

    for table in CLEAR_ORDER {
        let result = sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&state.pool)
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?;
        deleted += result.rows_affected();
    }

    Ok(deleted)
}
