use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicPeriod {
    pub id: String,
    pub title: String,
    pub start_on: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EllucianClass {
    pub section_id: String,
    pub crn: String,
    pub subject: String,
    pub number: String,
    pub title: String,
    pub term: String,
    pub term_title: String,
    pub moodle_shortname: Option<String>,
}

/// Weekly block of a section. `section_id`, `title` and `term` are filled in
/// by the client, because the schedule response does not carry the course.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionalEvent {
    pub id: String,
    pub section_id: Option<String>,
    pub title: Option<String>,
    pub term: Option<String>,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub room: Option<String>,
    pub building: Option<String>,
    pub instructor: Option<String>,
}
