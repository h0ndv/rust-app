use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicPeriod {
    pub id: String,
    pub title: String,
    /// Start of the term in Unix seconds.
    pub start_on: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EllucianClass {
    pub section_id: String,
    /// CRN of the section, for example `30061`.
    pub crn: String,
    pub subject: String,
    pub number: String,
    pub title: String,
    pub term: String,
    pub term_title: String,
    /// Moodle shortname if the extension exposes the AULA VIRTUAL link.
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
    /// 1 = Monday, 7 = Sunday.
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub room: Option<String>,
    pub building: Option<String>,
    pub instructor: Option<String>,
}
