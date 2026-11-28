use serde::{Deserialize, Serialize};

/// Wire mirror of `auth::Source`. The rich auth type keeps the URLs and the
/// readiness rules; this one is safe to send across the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Moodle,
    Ellucian,
    Banner,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatusKind {
    Running,
    #[default]
    Ok,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncRun {
    pub id: String,
    pub status: SyncStatusKind,
    pub requests: u32,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: SyncStatusKind,
    pub source: Option<Source>,
    pub progress: u8,
    pub requests: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskFilter {
    #[default]
    All,
    Overdue,
    Pending,
    Submitted,
    Graded,
    #[serde(rename = "no_data")]
    NoData,
    Unconfirmed,
    Informational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceFilter {
    #[default]
    All,
    New,
    Downloaded,
}

/// Entity kind behind a Novedades item. Typed at the command boundary so an
/// unknown value can never reach the acknowledgement query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NewsKind {
    Announcement,
    File,
    Activity,
}

impl NewsKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NewsKind::Announcement => "announcement",
            NewsKind::File => "file",
            NewsKind::Activity => "activity",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub course_id: String,
    pub course_name: String,
    pub title: String,
    pub due_at: Option<i64>,
    pub status: Option<String>,
    pub submitted: bool,
    pub grade: Option<f64>,
    pub url: Option<String>,
    pub is_new: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceDto {
    pub id: String,
    pub course_id: Option<String>,
    pub course: Option<String>,
    pub title: String,
    pub kind: String,
    pub mimetype: Option<String>,
    pub size: Option<i64>,
    pub url: String,
    pub is_new: bool,
    pub downloaded: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScheduleEventDto {
    pub id: String,
    pub course: Option<String>,
    pub title: String,
    pub day: Option<i64>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub room: Option<String>,
    pub building: Option<String>,
    pub instructor: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewsItemDto {
    pub id: String,
    pub course: Option<String>,
    pub title: String,
    pub body: String,
    pub published_at: Option<i64>,
    pub kind: String,
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DashboardDto {
    pub overdue: Vec<TaskDto>,
    pub due_soon: Vec<TaskDto>,
    pub next_classes: Vec<ScheduleEventDto>,
    pub new_resources: Vec<ResourceDto>,
}

impl DashboardDto {
    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GradeRowDto {
    pub course: String,
    pub name: String,
    pub grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GradeTotalDto {
    pub course: String,
    pub total: Option<f64>,
    pub max_total: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GradesDto {
    pub term: String,
    pub terms: Vec<String>,
    pub banner: Vec<GradeRowDto>,
    pub moodle_totals: Vec<GradeTotalDto>,
    pub gpa: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchHitDto {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub body: String,
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchResultsDto {
    pub query: String,
    pub total: usize,
    pub items: Vec<SearchHitDto>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecoveryNoticeDto {
    pub backup_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsDto {
    pub sync_interval_minutes: u32,
    pub downloads_dir: String,
    pub theme: String,
    pub notifications: bool,
    pub language: String,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            sync_interval_minutes: 30,
            downloads_dir: String::new(),
            theme: "system".into(),
            notifications: true,
            language: "en".into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CourseSectionDto {
    pub id: String,
    pub fullname: String,
    pub shortname: Option<String>,
    pub section: Option<String>,
    pub term: Option<String>,
    pub teacher: Option<String>,
    pub source: String,
    pub banner_crn: Option<String>,
    pub next_due_at: Option<i64>,
    pub pending_tasks: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CourseGroupDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub sections: Vec<CourseSectionDto>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActivityRow {
    pub id: String,
    pub name: String,
    pub modtype: String,
    pub status: Option<String>,
    pub due_at: Option<i64>,
    pub url: Option<String>,
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileRow {
    pub id: String,
    pub name: String,
    pub url: String,
    pub mimetype: Option<String>,
    pub size: Option<i64>,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GradeRow {
    pub id: String,
    pub name: String,
    pub grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScheduleRow {
    pub id: String,
    pub title: String,
    pub day_of_week: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub room: Option<String>,
    pub building: Option<String>,
    pub instructor: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CourseDetailDto {
    pub group: CourseGroupDto,
    pub tasks: Vec<ActivityRow>,
    pub resources: Vec<FileRow>,
    pub grades: Vec<GradeRow>,
    pub schedule: Vec<ScheduleRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Network,
    Unauthorized,
    Forbidden,
    Parse,
    NotFound,
    RateLimited,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppErrorPayload {
    pub kind: ErrorKind,
    pub message: String,
}
