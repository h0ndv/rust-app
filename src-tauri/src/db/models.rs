use serde::{Deserialize, Serialize};

/// Group of subjects by code, for example `IGR-00167`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CourseGroup {
    pub id: String,
    pub code: String,
    pub name: String,
}

/// A real section of a subject: lecture and lab are separate rows.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Course {
    pub id: String,
    pub group_id: Option<String>,
    pub source: String,
    pub moodle_id: Option<i64>,
    pub banner_crn: Option<String>,
    pub shortname: Option<String>,
    pub fullname: String,
    pub section: Option<String>,
    pub term: Option<String>,
    pub teacher: Option<String>,
    pub moodle_url: Option<String>,
    pub banner_url: Option<String>,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Section {
    pub id: String,
    pub course_id: String,
    pub name: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Activity {
    pub id: String,
    pub course_id: String,
    pub section_id: Option<String>,
    pub cmid: Option<i64>,
    pub name: String,
    pub modtype: String,
    pub url: Option<String>,
    pub due_at: Option<i64>,
    pub open_at: Option<i64>,
    pub cutoff_at: Option<i64>,
    pub status: Option<String>,
    pub submitted: Option<bool>,
    pub graded: Option<bool>,
    pub grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub description: String,
    pub content_hash: Option<String>,
    pub first_seen_at: Option<String>,
    pub acknowledged_at: Option<String>,
    pub archived_at: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileItem {
    pub id: String,
    pub activity_id: Option<String>,
    pub course_id: Option<String>,
    pub name: String,
    pub url: String,
    pub mimetype: Option<String>,
    pub size: Option<i64>,
    pub kind: String,
    pub sha256: Option<String>,
    pub local_path: Option<String>,
    pub first_seen_at: Option<String>,
    pub acknowledged_at: Option<String>,
    pub archived_at: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeItem {
    pub id: String,
    pub course_id: String,
    pub term: String,
    pub name: String,
    pub category: Option<String>,
    pub grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub weight: Option<f64>,
    pub contribution: Option<f64>,
    pub source: String,
    pub raw_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GradeTotal {
    pub course_id: String,
    pub term: String,
    pub total: Option<f64>,
    pub max_total: Option<f64>,
    pub source: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduleEvent {
    pub id: String,
    pub course_id: Option<String>,
    pub title: String,
    pub day_of_week: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub room: Option<String>,
    pub building: Option<String>,
    pub instructor: Option<String>,
    pub term: Option<String>,
    pub raw_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Announcement {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub body: String,
    pub url: Option<String>,
    pub published_at: Option<i64>,
    pub first_seen_at: Option<String>,
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncRun {
    pub id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub requests: i64,
    pub errors_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceStatus {
    pub source: String,
    pub state: String,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
}

/// Union computed between a Moodle course and a Banner course.
/// It is not a `course_links` row: the CRN lives in `courses.banner_crn`.
/// At least one of the two sides is present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourseLink {
    pub moodle_course_id: Option<String>,
    pub banner_course_id: Option<String>,
    pub banner_crn: Option<String>,
}
