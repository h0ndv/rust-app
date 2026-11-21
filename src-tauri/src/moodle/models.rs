use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodleCourse {
    pub id: i64,
    pub fullname: String,
    pub shortname: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodleEvent {
    pub name: String,
    pub modulename: String,
    pub instance: i64,
    pub timesort: i64,
    pub overdue: bool,
    pub url: Option<String>,
    pub course_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoodleSection {
    pub id: i64,
    pub name: String,
    pub activities: Vec<MoodleActivity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodleActivity {
    pub cmid: i64,
    pub name: String,
    pub modtype: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodleAssignment {
    pub submitted: bool,
    pub graded: bool,
    pub status_raw: String,
    pub due_at: Option<i64>,
    pub cutoff_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodleAnnouncement {
    pub id: Option<i64>,
    pub title: String,
    pub body: String,
    pub url: Option<String>,
    pub published_at: Option<i64>,
}

pub struct MoodleCourseContent {
    pub sections: Vec<MoodleSection>,
    pub announcements: Result<Vec<MoodleAnnouncement>, AppError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoodleGradeTotal {
    pub total: f64,
    pub max_total: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodleFile {
    pub name: String,
    pub url: String,
    pub mimetype: Option<String>,
    pub size: Option<i64>,
}
