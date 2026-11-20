use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BannerOption {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BannerGrade {
    pub subject: String,
    pub number: String,
    pub crn: String,
    pub section: String,
    pub title: String,
    pub final_grade: Option<String>,
    pub midterm_grade: Option<String>,
    pub history_final_grade: Option<String>,
    pub calculated_final_grade: Option<String>,
    pub term: String,
    pub term_title: String,
    pub level: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TermGpa {
    pub selected_term: Option<f64>,
    pub cumulative: Option<f64>,
}
