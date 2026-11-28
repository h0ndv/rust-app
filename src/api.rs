use std::future::Future;
use std::rc::Rc;

use contracts::{
    CourseDetailDto, CourseGroupDto, DashboardDto, GradesDto, NewsItemDto, NewsKind,
    RecoveryNoticeDto, ResourceDto, ResourceFilter, ScheduleEventDto, SearchResultsDto,
    SessionState, SettingsDto, Source, SyncStatus, TaskDto, TaskFilter,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::error::UiError;
use crate::transport::Transport;

#[derive(Clone)]
pub struct Api {
    transport: Rc<dyn Transport>,
}

impl Api {
    pub fn new(transport: Rc<dyn Transport>) -> Self {
        Self { transport }
    }

    fn call<T>(&self, command: &str, args: Value) -> impl Future<Output = Result<T, UiError>>
    where
        T: DeserializeOwned,
    {
        let transport = Rc::clone(&self.transport);
        let command = command.to_string();

        async move {
            let value = transport.invoke(&command, args).await?;
            serde_json::from_value(value)
                .map_err(|error| UiError::internal(format!("invalid {command} response: {error}")))
        }
    }

    pub fn connect(
        &self,
        source: Option<Source>,
    ) -> impl Future<Output = Result<SessionState, UiError>> {
        self.call("connect", json!({ "source": source }))
    }

    pub fn session_state(&self) -> impl Future<Output = Result<SessionState, UiError>> {
        self.call("session_state", json!({}))
    }

    pub fn refresh_session(&self) -> impl Future<Output = Result<SessionState, UiError>> {
        self.call("refresh_session", json!({}))
    }

    pub fn start_sync(&self) -> impl Future<Output = Result<SyncStatus, UiError>> {
        self.call("start_sync", json!({}))
    }

    pub fn cancel_sync(&self) -> impl Future<Output = Result<(), UiError>> {
        self.call("cancel_sync", json!({}))
    }

    pub fn sync_status(&self) -> impl Future<Output = Result<SyncStatus, UiError>> {
        self.call("sync_status", json!({}))
    }

    pub fn get_dashboard(&self) -> impl Future<Output = Result<DashboardDto, UiError>> {
        self.call("get_dashboard", json!({}))
    }

    pub fn has_local_data(&self) -> impl Future<Output = Result<bool, UiError>> {
        self.call("has_local_data", json!({}))
    }

    pub fn list_courses(&self) -> impl Future<Output = Result<Vec<CourseGroupDto>, UiError>> {
        self.call("list_courses", json!({}))
    }

    pub fn get_course(
        &self,
        group_id: &str,
    ) -> impl Future<Output = Result<CourseDetailDto, UiError>> {
        self.call("get_course", json!({ "groupId": group_id }))
    }

    pub fn list_tasks(
        &self,
        filter: TaskFilter,
    ) -> impl Future<Output = Result<Vec<TaskDto>, UiError>> {
        self.call("list_tasks", json!({ "filter": filter }))
    }

    pub fn list_resources(
        &self,
        filter: ResourceFilter,
    ) -> impl Future<Output = Result<Vec<ResourceDto>, UiError>> {
        self.call("list_resources", json!({ "filter": filter }))
    }

    pub fn list_grades(
        &self,
        term: Option<&str>,
    ) -> impl Future<Output = Result<GradesDto, UiError>> {
        self.call("list_grades", json!({ "term": term }))
    }

    pub fn get_schedule(&self) -> impl Future<Output = Result<Vec<ScheduleEventDto>, UiError>> {
        self.call("get_schedule", json!({}))
    }

    pub fn list_news(&self) -> impl Future<Output = Result<Vec<NewsItemDto>, UiError>> {
        self.call("list_news", json!({}))
    }

    pub fn acknowledge(
        &self,
        kind: NewsKind,
        id: &str,
    ) -> impl Future<Output = Result<(), UiError>> {
        self.call("acknowledge", json!({ "kind": kind, "id": id }))
    }

    pub fn search(&self, query: &str) -> impl Future<Output = Result<SearchResultsDto, UiError>> {
        self.call("search", json!({ "query": query }))
    }

    pub fn get_settings(&self) -> impl Future<Output = Result<SettingsDto, UiError>> {
        self.call("get_settings", json!({}))
    }

    pub fn recovery_notice(
        &self,
    ) -> impl Future<Output = Result<Option<RecoveryNoticeDto>, UiError>> {
        self.call("recovery_notice", json!({}))
    }

    pub fn save_settings(
        &self,
        settings: SettingsDto,
    ) -> impl Future<Output = Result<(), UiError>> {
        self.call("save_settings", json!({ "settings": settings }))
    }

    pub fn set_language(&self, language: &str) -> impl Future<Output = Result<(), UiError>> {
        self.call("set_language", json!({ "language": language }))
    }

    pub fn open_in_app(&self, url: &str) -> impl Future<Output = Result<(), UiError>> {
        self.call("open_in_app", json!({ "url": url }))
    }

    pub fn download_file(&self, file_id: &str) -> impl Future<Output = Result<(), UiError>> {
        self.call("download_file", json!({ "fileId": file_id }))
    }

    pub fn clear_data(&self) -> impl Future<Output = Result<u64, UiError>> {
        self.call("clear_data", json!({}))
    }

    pub fn export_data(&self) -> impl Future<Output = Result<String, UiError>> {
        self.call("export_data", json!({}))
    }

    pub fn clear_downloads(&self) -> impl Future<Output = Result<u64, UiError>> {
        self.call("clear_downloads", json!({}))
    }

    pub fn set_window_theme(&self, theme: &str) -> impl Future<Output = Result<(), UiError>> {
        self.call("set_window_theme", json!({ "theme": theme }))
    }

    pub fn notification_permission(&self) -> impl Future<Output = Result<bool, UiError>> {
        self.call("notification_permission", json!({}))
    }

    pub fn request_notification_permission(&self) -> impl Future<Output = Result<bool, UiError>> {
        self.call("request_notification_permission", json!({}))
    }

    pub fn send_notification(
        &self,
        title: &str,
        body: &str,
    ) -> impl Future<Output = Result<(), UiError>> {
        self.call("send_notification", json!({ "title": title, "body": body }))
    }
}

pub fn use_api() -> Api {
    crate::state::use_app().api
}
