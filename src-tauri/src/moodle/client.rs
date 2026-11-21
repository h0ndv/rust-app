use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::Mutex;

use futures::stream::{self, StreamExt, TryStreamExt};

use crate::auth::{SessionManager, Source};
use crate::error::AppError;
use crate::http::HttpClient;
use crate::moodle::models::{
    MoodleAnnouncement, MoodleAssignment, MoodleCourse, MoodleCourseContent, MoodleEvent,
    MoodleGradeTotal, MoodleSection,
};
use crate::moodle::parser;

pub const BASE_URL: &str = "https://aulasvirtuales.santotomas.cl";

/// Moodle client. Read only: it never submits, edits or deletes.
pub struct MoodleClient {
    http: HttpClient,
    session: Arc<SessionManager>,
    sesskey: Mutex<Option<String>>,
}

impl MoodleClient {
    pub fn new(http: HttpClient, session: Arc<SessionManager>) -> Self {
        Self {
            http,
            session,
            sesskey: Mutex::new(None),
        }
    }

    async fn cookies(&self) -> Result<String, AppError> {
        self.session.cookie_header(Source::Moodle).await
    }

    async fn get_url(&self, url: &str) -> Result<reqwest::Response, AppError> {
        let cookies = self.cookies().await?;
        let response = self.http.get(url, &cookies).await?;
        if is_login_page(response.url().path()) {
            return Err(AppError::Unauthorized);
        }
        Ok(response)
    }

    async fn get_text(&self, url: &str) -> Result<String, AppError> {
        let response = self.get_url(url).await?;

        // A `resource` redirects to the file: reading a PDF as text hangs the
        // run without adding anything.
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_lowercase();
        if !content_type.is_empty() && !content_type.contains("html") {
            return Ok(String::new());
        }

        crate::http::read_text(response).await
    }

    async fn get_path(&self, path: &str) -> Result<String, AppError> {
        self.get_text(&format!("{BASE_URL}{path}")).await
    }

    /// The `sesskey` rarely changes and every AJAX call needs it: it is kept
    /// in memory to avoid requesting the courses page on every query.
    async fn sesskey(&self) -> Result<String, AppError> {
        if let Some(key) = self.sesskey.lock().await.clone() {
            return Ok(key);
        }

        let html = self.get_path("/my/courses.php").await?;
        let regex = regex::Regex::new(r#""sesskey":"([^"]+)""#).expect("regex válida");
        let key = regex
            .captures(&html)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or(AppError::Unauthorized)?;

        *self.sesskey.lock().await = Some(key.clone());
        Ok(key)
    }

    async fn ajax(&self, method: &str, args: Value) -> Result<String, AppError> {
        let key = self.sesskey().await?;
        let url = format!("{BASE_URL}/lib/ajax/service.php?sesskey={key}&info={method}");
        let body = json!([{ "index": 0, "methodname": method, "args": args }]);
        let cookies = self.cookies().await?;

        let response = self.http.post_json(&url, &cookies, &body).await?;
        if is_login_page(response.url().path()) {
            return Err(AppError::Unauthorized);
        }
        let text = response
            .text()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        if text.trim_start().starts_with('<') {
            return Err(AppError::Unauthorized);
        }
        Ok(text)
    }

    pub async fn fetch_courses(&self) -> Result<Vec<MoodleCourse>, AppError> {
        let body = self
            .ajax(
                "core_course_get_enrolled_courses_by_timeline_classification",
                json!({
                    "offset": 0,
                    "limit": 0,
                    "classification": "all",
                    "sort": "fullname",
                    "customfieldname": "",
                    "customfieldvalue": ""
                }),
            )
            .await?;
        parser::parse_courses_json(&body)
    }

    pub async fn fetch_calendar(&self) -> Result<Vec<MoodleEvent>, AppError> {
        let now = chrono::Utc::now().timestamp();
        let body = self
            .ajax(
                "core_calendar_get_action_events_by_timesort",
                json!({
                    "limitnum": 50,
                    "timesortfrom": now - 180 * 24 * 60 * 60,
                    "timesortto": now + 180 * 24 * 60 * 60,
                    "limittononsuspendedevents": true
                }),
            )
            .await?;
        parser::parse_calendar_json(&body)
    }

    pub async fn fetch_sections(&self, course_id: i64) -> Result<Vec<MoodleSection>, AppError> {
        let first_page = self
            .get_path(&format!("/course/view.php?id={course_id}"))
            .await?;
        self.parse_sections(course_id, &first_page).await
    }

    pub async fn fetch_course_content(
        &self,
        course_id: i64,
    ) -> Result<MoodleCourseContent, AppError> {
        let first_page = self
            .get_path(&format!("/course/view.php?id={course_id}"))
            .await?;
        let sections = self.parse_sections(course_id, &first_page).await?;
        let announcements = self.parse_announcements(&first_page).await;

        Ok(MoodleCourseContent {
            sections,
            announcements,
        })
    }

    async fn parse_sections(
        &self,
        course_id: i64,
        first_page: &str,
    ) -> Result<Vec<MoodleSection>, AppError> {
        // Matches the HTTP client's in-flight limit so the pages do not
        // queue behind a wider burst.
        const CONCURRENCY: usize = 4;

        let numbers = parser::parse_section_numbers(first_page);

        let mut sections: Vec<(usize, MoodleSection)> =
            stream::iter(numbers.iter().copied().enumerate())
                .map(|(index, number)| async move {
                    let html = if number == 0 {
                        first_page.to_string()
                    } else {
                        self.get_path(&format!("/course/view.php?id={course_id}&section={number}"))
                            .await?
                    };
                    let section = parser::parse_course_html(&html, number)?;
                    Ok::<_, AppError>((index, section))
                })
                .buffer_unordered(CONCURRENCY)
                .try_collect()
                .await?;

        sections.sort_by_key(|(index, _)| *index);
        Ok(sections.into_iter().map(|(_, section)| section).collect())
    }

    async fn parse_announcements(
        &self,
        first_page: &str,
    ) -> Result<Vec<MoodleAnnouncement>, AppError> {
        let Some(forum_url) = parser::parse_news_forum_url(first_page) else {
            return Ok(Vec::new());
        };

        let page = self.get_text(&forum_url).await?;
        parser::parse_announcements_html(&page)
    }

    pub async fn fetch_assignment(&self, cmid: i64) -> Result<MoodleAssignment, AppError> {
        let html = self
            .get_path(&format!("/mod/assign/view.php?id={cmid}"))
            .await?;
        parser::parse_assignment_html(&html)
    }

    pub async fn fetch_page(&self, url: &str) -> Result<String, AppError> {
        self.get_text(url).await
    }

    pub async fn download_file(&self, url: &str, dest: &Path) -> Result<PathBuf, AppError> {
        let response = self.get_url(url).await?;
        let bytes = crate::http::read_bytes(response).await?;

        if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Internal(format!("no se pudo crear la carpeta: {e}")))?;
        }
        tokio::fs::write(dest, &bytes).await.map_err(|e| {
            AppError::Internal(format!("no se pudo guardar {}: {e}", dest.display()))
        })?;

        Ok(dest.to_path_buf())
    }

    pub async fn fetch_announcements(
        &self,
        course_id: i64,
    ) -> Result<Vec<MoodleAnnouncement>, AppError> {
        let html = self
            .get_path(&format!("/course/view.php?id={course_id}"))
            .await?;
        self.parse_announcements(&html).await
    }

    /// The per-user report may not exist in this installation: with no
    /// total it returns `None` and the sync continues.
    pub async fn fetch_grade_total(
        &self,
        course_id: i64,
    ) -> Result<Option<MoodleGradeTotal>, AppError> {
        let html = match self
            .get_path(&format!("/grade/report/user/index.php?id={course_id}"))
            .await
        {
            Ok(html) => html,
            Err(AppError::NotFound) => return Ok(None),
            Err(error) => return Err(error),
        };

        match parser::parse_grade_total_html(&html) {
            Ok(total) => Ok(Some(total)),
            Err(AppError::Parse(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

fn is_login_page(path: &str) -> bool {
    path.contains("/login/index.php")
}
