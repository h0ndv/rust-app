pub mod auth;
pub mod banner;
pub mod commands;
pub mod db;
pub mod ellucian;
pub mod error;
pub mod http;
pub mod moodle;
pub mod state;
pub mod sync;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tracing_subscriber::prelude::*;

use auth::{SessionManager, Source};
use banner::BannerClient;
use db::repo::Repo;
use ellucian::EllucianClient;
use http::HttpClient;
use moodle::MoodleClient;
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
            let session = Arc::new(SessionManager::new(handle.clone()));
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
                emitter: Some(tauri_emitter(handle.clone())),
            }));
            sync.spawn_scheduled(Duration::from_secs(30 * 60));

            let folder = downloads_folder(&data_dir);
            let downloads = Downloads::new(
                repo.clone(),
                live,
                folder,
                Some(tauri_emitter(handle.clone())),
            );

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
            open_login,
            open_moodle,
            open_banner,
            smoke_moodle,
            smoke_ellucian,
            smoke_banner,
            smoke_sync,
            commands::connect,
            commands::session_state,
            commands::start_sync,
            commands::cancel_sync,
            commands::sync_status,
            commands::get_dashboard,
            commands::list_courses,
            commands::get_course,
            commands::list_tasks,
            commands::list_resources,
            commands::list_grades,
            commands::get_schedule,
            commands::list_news,
            commands::search,
            commands::get_settings,
            commands::save_settings,
            commands::open_in_app,
            commands::download_file,
            commands::clear_data
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar rust-app");
}

fn downloads_folder(data_dir: &std::path::Path) -> PathBuf {
    std::env::var("USERPROFILE")
        .map(|home| PathBuf::from(home).join("Downloads").join("PanelRamos"))
        .unwrap_or_else(|_| data_dir.join("downloads"))
}

/// The emitter lives in the binary: the lib does not import the Tauri symbols
/// that drag WebView2 into the tests.
fn tauri_emitter(app: AppHandle) -> sync::EventEmitter {
    use tauri::Emitter;
    Arc::new(move |event: &str, payload: serde_json::Value| {
        let _ = app.emit(event, payload);
    })
}

/// Temporary harness for Task 4: it checks that opening windows from async
/// commands does not hang the app. Onboarding replaces it in Task 11.
#[tauri::command]
async fn open_login(app: AppHandle) -> Result<(), String> {
    auth::windows::open_auth_window(&app, Source::Ellucian)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn open_moodle(app: AppHandle) -> Result<(), String> {
    auth::windows::open_auth_window(&app, Source::Moodle)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn open_banner(app: AppHandle) -> Result<(), String> {
    auth::windows::open_auth_window(&app, Source::Banner)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Temporary harness for Tasks 5 and 6: it tests the clients with the real session.
#[tauri::command]
async fn smoke_moodle(session: tauri::State<'_, Arc<SessionManager>>) -> Result<String, String> {
    let client = MoodleClient::new(
        HttpClient::new(auth::USER_AGENT),
        Arc::clone(session.inner()),
    );
    let courses = client
        .fetch_courses()
        .await
        .map_err(|error| error.to_string())?;
    Ok(format!("{} cursos en Moodle", courses.len()))
}

#[tauri::command]
async fn smoke_ellucian(session: tauri::State<'_, Arc<SessionManager>>) -> Result<String, String> {
    let client = EllucianClient::new(
        HttpClient::new(auth::USER_AGENT),
        Arc::clone(session.inner()),
    );
    let periods = client
        .fetch_periods()
        .await
        .map_err(|error| error.to_string())?;
    let classes = client
        .fetch_classes()
        .await
        .map_err(|error| error.to_string())?;
    let schedule = client
        .fetch_schedule_for(&classes)
        .await
        .map_err(|error| error.to_string())?;
    Ok(format!(
        "{} periodos, {} clases, {} bloques de horario",
        periods.len(),
        classes.len(),
        schedule.len()
    ))
}

#[tauri::command]
async fn smoke_banner(session: tauri::State<'_, Arc<SessionManager>>) -> Result<String, String> {
    let client = BannerClient::new(
        HttpClient::new(auth::USER_AGENT),
        Arc::clone(session.inner()),
    );
    let terms = client.terms().await.map_err(|error| error.to_string())?;
    let term = terms
        .iter()
        .find(|term| term.code != "-1")
        .map(|term| term.code.clone())
        .ok_or_else(|| "Banner no devolvió periodos".to_string())?;
    let levels = client
        .levels(&term)
        .await
        .map_err(|error| error.to_string())?;
    let level = levels
        .first()
        .map(|level| level.code.clone())
        .ok_or_else(|| "Banner no devolvió niveles".to_string())?;
    let grades = client
        .courses(&term, &level, "-2")
        .await
        .map_err(|error| error.to_string())?;
    let gpa = client.gpa(&term).await.map_err(|error| error.to_string())?;
    Ok(format!(
        "{} periodos, {} niveles, {} ramos en {}, GPA por periodo {:?}",
        terms.len(),
        levels.len(),
        grades.len(),
        term,
        gpa.selected_term
    ))
}

/// Temporary harness for Task 9: it runs a full sync with the
/// real session. Task 10 replaces it.
#[tauri::command]
async fn smoke_sync(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let run = state
        .sync
        .run(sync::SyncTrigger::Manual)
        .await
        .map_err(|error| error.to_string())?;
    let groups = state
        .repo
        .list_course_groups()
        .await
        .map_err(|error| error.to_string())?;
    let activities: i64 = sqlx::query_scalar("SELECT count(*) FROM activities")
        .fetch_one(&state.pool)
        .await
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "{:?}: {} peticiones, {} grupos, {} actividades",
        run.status,
        run.requests,
        groups.len(),
        activities
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}
