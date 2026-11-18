use serde::{Deserialize, Serialize};

/// Filters sent by the interface. `lowercase` so the JSON is `"overdue"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskFilter {
    #[default]
    All,
    Overdue,
    Pending,
    Submitted,
    Graded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResourceFilter {
    #[default]
    All,
    New,
    Downloaded,
}

#[derive(Debug, Clone, Default, Serialize)]
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
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ResourceDto {
    pub id: String,
    pub course_id: Option<String>,
    pub title: String,
    pub kind: String,
    pub mimetype: Option<String>,
    pub size: Option<i64>,
    pub url: String,
    pub is_new: bool,
    pub downloaded: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct NewsItemDto {
    pub id: String,
    pub course: Option<String>,
    pub title: String,
    pub body: String,
    pub published_at: Option<i64>,
    pub kind: String,
}

#[derive(Debug, Clone, Default, Serialize)]
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct GradeRowDto {
    pub course: String,
    pub name: String,
    pub grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GradeTotalDto {
    pub course: String,
    pub total: Option<f64>,
    pub max_total: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GradesDto {
    pub term: String,
    pub terms: Vec<String>,
    pub banner: Vec<GradeRowDto>,
    pub moodle_totals: Vec<GradeTotalDto>,
    pub gpa: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchHitDto {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchResultsDto {
    pub query: String,
    pub total: usize,
    pub items: Vec<SearchHitDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub sync_interval_minutes: u32,
    pub downloads_dir: String,
    pub theme: String,
    pub notifications: bool,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            sync_interval_minutes: 30,
            downloads_dir: String::new(),
            theme: "system".into(),
            notifications: true,
        }
    }
}
