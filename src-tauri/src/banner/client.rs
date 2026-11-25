use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::Url;

use crate::auth::{SessionManager, Source};
use crate::banner::eval;
use crate::banner::models::{BannerGrade, BannerOption, TermGpa};
use crate::banner::parser;
use crate::error::AppError;
use crate::http::HttpClient;

pub const BANNER_BASE: &str =
    "https://selferviceprd.santotomas.cl/StudentSelfService/studentGrades";
const WINDOW: &str = "banner";

pub struct BannerClient {
    http: HttpClient,
    session: Arc<SessionManager>,
}

impl BannerClient {
    pub fn new(http: HttpClient, session: Arc<SessionManager>) -> Self {
        Self { http, session }
    }

    pub async fn terms(&self) -> Result<Vec<BannerOption>, AppError> {
        parser::parse_options_response(&self.fetch_json("term").await?)
    }

    pub async fn levels(&self, term: &str) -> Result<Vec<BannerOption>, AppError> {
        parser::parse_options_response(&self.fetch_json(&format!("level?term={term}")).await?)
    }

    pub async fn study_paths(
        &self,
        term: &str,
        level: &str,
    ) -> Result<Vec<BannerOption>, AppError> {
        parser::parse_options_response(
            &self
                .fetch_json(&format!("studyPath?term={term}&level={level}"))
                .await?,
        )
    }

    pub async fn courses(
        &self,
        term: &str,
        level: &str,
        study_path: &str,
    ) -> Result<Vec<BannerGrade>, AppError> {
        let path = format!(
            "courses?termCode={term}&levelCode={level}&studyPathCode={study_path}&pageOffset=0&pageMaxSize=25"
        );
        parser::parse_courses_response(&self.fetch_json(&path).await?)
    }

    pub async fn gpa(&self, term: &str) -> Result<TermGpa, AppError> {
        parser::parse_gpa_response(&self.fetch_json(&format!("gpa?term={term}")).await?)
    }

    /// `reqwest` works for some endpoints; when Banner returns the
    /// SSO page the request is retried inside the authenticated window.
    async fn fetch_json(&self, relative: &str) -> Result<String, AppError> {
        let url = format!("{BANNER_BASE}/{relative}");
        let cookies = self.session.cookie_header(Source::Banner).await?;

        match self.http.get(&url, &cookies).await {
            Ok(response) => {
                let text = crate::http::read_text(response).await?;
                if parser::looks_like_sso(&text) {
                    self.fetch_in_window(&url).await
                } else {
                    Ok(text)
                }
            }
            Err(AppError::Unauthorized) => self.fetch_in_window(&url).await,
            Err(error) => Err(error),
        }
    }

    async fn fetch_in_window(&self, url: &str) -> Result<String, AppError> {
        let window = self.window().await?;
        let script = format!(
            r#"
(async () => {{
  const response = await fetch('{url}', {{
    credentials: 'include',
    headers: {{ 'Accept': 'application/json', 'X-Requested-With': 'XMLHttpRequest' }}
  }});
  const text = await response.text();
  return JSON.stringify({{ status: response.status, body: text }});
}})()
"#
        );

        let payload: Value = eval::eval_json(&window, &script).await?;
        let status = payload.get("status").and_then(Value::as_u64).unwrap_or(0);
        let body = payload
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        if matches!(status, 401 | 403) || parser::looks_like_sso(&body) {
            return Err(AppError::Unauthorized);
        }
        Ok(body)
    }

    /// Hidden window, reused between queries. The first call has to wait for the
    /// CAS chain to reach the grades page.
    async fn window(&self) -> Result<WebviewWindow, AppError> {
        let app = self.session.app();
        if let Some(window) = app.get_webview_window(WINDOW) {
            return Ok(window);
        }

        let url = Url::parse(BANNER_BASE)
            .map_err(|e| AppError::Internal(format!("invalid Banner URL: {e}")))?;

        let window = WebviewWindowBuilder::new(&app, WINDOW, WebviewUrl::External(url))
            .title("Banner")
            .inner_size(1024.0, 768.0)
            .visible(false)
            .build()
            .map_err(|e| AppError::Internal(format!("could not open Banner: {e}")))?;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
        loop {
            if let Ok(current) = window.url()
                && Source::Banner.is_ready(&current)
            {
                return Ok(window);
            }
            if tokio::time::Instant::now() >= deadline {
                let _ = window.destroy();
                return Err(AppError::Network(
                    "Banner did not finish authentication within 2 minutes".into(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
