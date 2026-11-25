pub mod auth;
pub mod banner;
pub mod commands;
pub mod db;
pub mod ellucian;
pub mod error;
pub mod events;
pub mod http;
pub mod moodle;
pub mod state;
pub mod sync;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tracing_subscriber::prelude::*;

use auth::SessionManager;
use db::repo::Repo;
use http::HttpClient;
use state::AppState;
use sync::{Downloads, LiveSources, SyncDeps, SyncOrchestrator};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let emitter = tauri_emitter(handle.clone());
            let session = Arc::new(SessionManager::new(
                handle.clone(),
                Some(Arc::clone(&emitter)),
            ));
            session.spawn_health_loop(Duration::from_secs(15 * 60));
            app.manage(Arc::clone(&session));

            let data_dir = app.path().app_data_dir()?;
            let pool = tauri::async_runtime::block_on(db::connect(&data_dir.join("panel.db")))?;
            tauri::async_runtime::block_on(db::migrate(&pool))?;

            let repo = Repo::new(pool.clone());
            let http = HttpClient::new(auth::USER_AGENT);
            let live = Arc::new(LiveSources::new(http.clone(), Arc::clone(&session)));

            let sync = Arc::new(SyncOrchestrator::new(SyncDeps {
                moodle: live.clone(),
                ellucian: live.clone(),
                banner: live.clone(),
                repo: repo.clone(),
                limiter: http.limiter(),
                emitter: Some(Arc::clone(&emitter)),
            }));
            sync.spawn_scheduler();

            let folder = downloads_folder(&data_dir);
            let downloads = Downloads::new(repo.clone(), live, folder, Some(emitter));

            app.manage(AppState::new(pool, repo, Arc::clone(&sync)));
            app.manage(sync);
            app.manage(downloads);

            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Focused(true))
                && window.label() == auth::MAIN_WINDOW
                && let Some(session) = window.app_handle().try_state::<Arc<SessionManager>>()
            {
                let session = Arc::clone(session.inner());
                tauri::async_runtime::spawn(async move { session.refresh_on_focus().await });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::session_state,
            commands::refresh_session,
            commands::start_sync,
            commands::cancel_sync,
            commands::sync_status,
            commands::get_dashboard,
            commands::has_local_data,
            commands::list_courses,
            commands::get_course,
            commands::list_tasks,
            commands::list_resources,
            commands::list_grades,
            commands::get_schedule,
            commands::list_news,
            commands::acknowledge,
            commands::search,
            commands::get_settings,
            commands::save_settings,
            commands::set_language,
            commands::open_in_app,
            commands::download_file,
            commands::clear_data
        ])
        .run(tauri::generate_context!())
        .expect("failed to start rust-app");
}

fn downloads_folder(data_dir: &std::path::Path) -> PathBuf {
    std::env::var("USERPROFILE")
        .map(|home| PathBuf::from(home).join("Downloads").join("rust-app"))
        .unwrap_or_else(|_| data_dir.join("downloads"))
}

/// The emitter lives in the binary: the lib does not import the Tauri symbols
/// that drag WebView2 into the tests.
fn tauri_emitter(app: AppHandle) -> events::EventEmitter {
    use tauri::Emitter;
    Arc::new(move |event: &str, payload: serde_json::Value| {
        let _ = app.emit(event, payload);
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}
