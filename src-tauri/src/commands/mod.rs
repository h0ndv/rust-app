pub mod auth;
pub mod data;
pub mod dto;
pub mod sync;

pub use auth::*;
pub use data::*;
pub use dto::*;
pub use sync::*;

use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use url::Url;

use crate::state::AppState;
use crate::sync::Downloads;

/// A single reused `browser` window: it keeps the profile session.
#[tauri::command]
pub async fn open_in_app(app: AppHandle, url: String) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|error| error.to_string())?;

    let window = match app.get_webview_window("browser") {
        Some(window) => window,
        None => WebviewWindowBuilder::new(&app, "browser", WebviewUrl::External(parsed.clone()))
            .title("rust-app")
            .inner_size(1100.0, 800.0)
            .build()
            .map_err(|error| error.to_string())?,
    };

    window.navigate(parsed).map_err(|error| error.to_string())?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
pub fn download_file(downloads: State<'_, Downloads>, file_id: String) -> Result<(), String> {
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

/// Deletes the local data. The WebView2 profile is not touched while the app
/// is open; that is left for the Settings screen in Task 12.
#[tauri::command]
pub async fn clear_data(state: State<'_, AppState>) -> Result<u64, String> {
    let mut deleted = 0u64;

    for table in CLEAR_ORDER {
        let result = sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&state.pool)
            .await
            .map_err(|error| error.to_string())?;
        deleted += result.rows_affected();
    }

    Ok(deleted)
}
