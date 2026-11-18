use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::Url;

use crate::auth::{AUTH_WINDOW, Source};
use crate::error::AppError;

const STEP_TIMEOUT: Duration = Duration::from_secs(300);
const POLL: Duration = Duration::from_millis(400);

/// Moodle does not redirect to SSO on its own: the OAuth2 link must be clicked.
const CLICK_OAUTH: &str = r#"
(() => {
  const link = [...document.querySelectorAll('a')]
    .find((a) => (a.href || '').includes('/auth/oauth2/login.php'));
  if (link) link.click();
})();
"#;

/// Opens (or reuses) the `auth` window and waits for the SSO chain to
/// finish. It must be called from an `async` command: creating the window on
/// the main thread hangs the app.
pub async fn open_auth_window(app: &AppHandle, source: Source) -> Result<WebviewWindow, AppError> {
    let window = match app.get_webview_window(AUTH_WINDOW) {
        Some(window) => window,
        None => {
            let start = parse_url(source.start_url())?;
            WebviewWindowBuilder::new(app, AUTH_WINDOW, WebviewUrl::External(start))
                .title("Conectar con Microsoft")
                .inner_size(1024.0, 768.0)
                .build()
                .map_err(|e| {
                    AppError::Internal(format!("no se pudo abrir la ventana de login: {e}"))
                })?
        }
    };

    window
        .navigate(parse_url(source.start_url())?)
        .map_err(|e| AppError::Internal(format!("no se pudo navegar: {e}")))?;
    let _ = window.show();
    let _ = window.set_focus();

    wait_until_ready(&window, source).await?;
    Ok(window)
}

async fn wait_until_ready(window: &WebviewWindow, source: Source) -> Result<(), AppError> {
    let deadline = tokio::time::Instant::now() + STEP_TIMEOUT;
    let mut clicked_for: Option<String> = None;

    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::Internal(format!(
                "{}: el login no terminó a tiempo",
                source.label()
            )));
        }

        let current = window
            .url()
            .map_err(|e| AppError::Internal(format!("no se pudo leer la URL: {e}")))?;

        if source.is_ready(&current) {
            return Ok(());
        }

        if source == Source::Moodle && current.path().contains("/login/index.php") {
            let key = current.as_str().to_string();
            if clicked_for.as_deref() != Some(key.as_str()) {
                let _ = window.eval(CLICK_OAUTH);
                clicked_for = Some(key);
            }
        }

        tokio::time::sleep(POLL).await;
    }
}

fn parse_url(value: &str) -> Result<Url, AppError> {
    Url::parse(value).map_err(|e| AppError::Internal(format!("URL inválida {value}: {e}")))
}
