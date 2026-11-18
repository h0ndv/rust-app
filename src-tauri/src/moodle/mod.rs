pub mod client;
pub mod models;
pub mod parser;

pub use client::MoodleClient;
pub use models::{
    MoodleActivity, MoodleAnnouncement, MoodleAssignment, MoodleCourse, MoodleEvent,
    MoodleGradeTotal, MoodleSection,
};
