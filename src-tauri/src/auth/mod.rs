pub mod cookies;
pub mod health;
pub mod windows;

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};
use tokio::sync::Mutex;
use tokio::time::Instant;
use url::Url;

use crate::error::AppError;
use crate::http::HttpClient;

pub const USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ",
    "(KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 PanelRamos/0.1"
);

pub const AUTH_WINDOW: &str = "auth";
pub const MAIN_WINDOW: &str = "main";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Moodle,
    Ellucian,
    Banner,
}

impl Source {
    /// Connection order: Ellucian opens the SAML chain, Moodle and Banner
    /// reuse the Microsoft session that stays alive.
    pub const ALL: [Source; 3] = [Source::Ellucian, Source::Moodle, Source::Banner];

    pub fn as_str(self) -> &'static str {
        match self {
            Source::Moodle => "moodle",
            Source::Ellucian => "ellucian",
            Source::Banner => "banner",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Source::Moodle => "Moodle",
            Source::Ellucian => "Ellucian",
            Source::Banner => "Banner",
        }
    }

    pub fn start_url(self) -> &'static str {
        match self {
            Source::Ellucian => "https://portal.santotomas.cl/",
            Source::Moodle => "https://aulasvirtuales.santotomas.cl/my/",
            Source::Banner => {
                "https://selferviceprd.santotomas.cl/StudentSelfService/studentGrades"
            }
        }
    }

    pub fn cookie_url(self) -> &'static str {
        match self {
            Source::Ellucian => "https://experience.elluciancloud.com/ustc/",
            Source::Moodle => "https://aulasvirtuales.santotomas.cl/my/",
            Source::Banner => {
                "https://selferviceprd.santotomas.cl/StudentSelfService/studentGrades"
            }
        }
    }

    pub fn is_ready(self, url: &Url) -> bool {
        match self {
            Source::Ellucian => {
                url.host_str() == Some("experience.elluciancloud.com")
                    && matches!(url.path(), "/ustc/" | "/ustc")
            }
            Source::Moodle => url.path().trim_matches('/') == "my",
            // This installation serves the page without the `ssb` segment that
            // the spike documented.
            Source::Banner => {
                url.path().contains("/StudentSelfService")
                    && url.path().trim_end_matches('/').ends_with("studentGrades")
            }
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "source", rename_all = "lowercase")]
pub enum SessionState {
    Disconnected,
    Connecting(Source),
    Connected,
    Expired(Source),
    Partial,
}

impl SessionState {
    pub fn connecting(self, source: Source) -> Self {
        SessionState::Connecting(source)
    }

    /// A healthy source moves the session to `Connected`, unless there is already a
    /// failure: a partial source does not recover because another one answers.
    pub fn source_ok(self, _source: Source) -> Self {
        match self {
            SessionState::Partial => SessionState::Partial,
            _ => SessionState::Connected,
        }
    }

    pub fn source_failed(self, _source: Source) -> Self {
        SessionState::Partial
    }

    /// Final state of a full connection: `Connected` only if no
    /// source failed. The transitions do not work for this because
    /// `Connecting(next)` does not remember a previous failure.
    pub fn outcome(failed_sources: usize) -> Self {
        if failed_sources == 0 {
            SessionState::Connected
        } else {
            SessionState::Partial
        }
    }
}

pub struct SessionManager {
    app: AppHandle,
    http: HttpClient,
    state: Mutex<SessionState>,
    last_focus_check: Mutex<Instant>,
}

impl SessionManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            http: HttpClient::new(USER_AGENT),
            state: Mutex::new(SessionState::Disconnected),
            last_focus_check: Mutex::new(Instant::now()),
        }
    }

    pub async fn state(&self) -> SessionState {
        self.state.lock().await.clone()
    }

    pub fn app(&self) -> AppHandle {
        self.app.clone()
    }

    pub async fn connect_all(&self) -> Result<SessionState, AppError> {
        let mut failed = 0usize;

        for source in Source::ALL {
            match self.authenticate(source).await {
                Ok(()) => {}
                Err(error) => {
                    failed += 1;
                    tracing::warn!(source = source.as_str(), %error, "fuente sin sesión");
                }
            }
        }

        let state = SessionState::outcome(failed);
        *self.state.lock().await = state.clone();
        Ok(state)
    }

    pub async fn ensure(&self, source: Source) -> Result<(), AppError> {
        if self.health(source).await? {
            return Ok(());
        }

        self.authenticate(source).await?;
        let next = {
            let current = self.state.lock().await;
            current.clone().source_ok(source)
        };
        *self.state.lock().await = next;
        Ok(())
    }

    /// `Cookie` header ready for `reqwest`. It is never stored or logged.
    pub async fn cookie_header(&self, source: Source) -> Result<String, AppError> {
        let window = self.window()?;
        cookies::header_for(&window, source.cookie_url()).await
    }

    pub async fn health(&self, source: Source) -> Result<bool, AppError> {
        let cookies = self.cookie_header(source).await?;
        health::check(&self.http, source, &cookies).await
    }

    async fn authenticate(&self, source: Source) -> Result<(), AppError> {
        if self.health(source).await.unwrap_or(false) {
            return Ok(());
        }

        let window = windows::open_auth_window(&self.app, source).await?;

        let result = async {
            let header = cookies::header_for(&window, source.cookie_url()).await?;
            if header.is_empty() {
                return Err(AppError::Unauthorized);
            }
            if self.health(source).await? {
                Ok(())
            } else {
                Err(AppError::Unauthorized)
            }
        }
        .await;

        let _ = window.hide();
        let _ = window.destroy();
        result
    }

    fn window(&self) -> Result<WebviewWindow, AppError> {
        self.app
            .get_webview_window(AUTH_WINDOW)
            .or_else(|| self.app.get_webview_window(MAIN_WINDOW))
            .ok_or_else(|| AppError::Internal("no hay ventana para leer cookies".into()))
    }

    pub async fn refresh_health(&self) -> SessionState {
        let mut down = Vec::new();
        for source in Source::ALL {
            if !matches!(self.health(source).await, Ok(true)) {
                down.push(source);
            }
        }

        let previous = self.state.lock().await.clone();
        let next = if down.is_empty() {
            SessionState::Connected
        } else if down.len() < Source::ALL.len() {
            SessionState::Partial
        } else {
            match previous {
                SessionState::Connected | SessionState::Partial | SessionState::Connecting(_) => {
                    SessionState::Expired(down[0])
                }
                _ => SessionState::Disconnected,
            }
        };

        *self.state.lock().await = next.clone();
        next
    }

    /// Check on focus regain, throttled so the servers are not hit every time
    /// the window goes and comes back.
    pub async fn refresh_on_focus(&self) {
        const DEBOUNCE: Duration = Duration::from_secs(60);

        {
            let mut last = self.last_focus_check.lock().await;
            if last.elapsed() < DEBOUNCE {
                return;
            }
            *last = Instant::now();
        }

        self.refresh_health().await;
    }

    /// Every 15 minutes while the app is open. Uses the Tauri runtime:
    /// `setup` runs on the main thread, with no Tokio reactor.
    pub fn spawn_health_loop(
        self: &Arc<Self>,
        every: Duration,
    ) -> tauri::async_runtime::JoinHandle<()> {
        let manager = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(every);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            ticker.tick().await;

            loop {
                ticker.tick().await;
                let state = manager.refresh_health().await;
                tracing::debug!(?state, "chequeo de sesión");
            }
        })
    }
}
