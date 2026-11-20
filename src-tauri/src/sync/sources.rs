use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use crate::auth::{SessionManager, Source};
use crate::banner::BannerClient;
use crate::banner::models::{BannerGrade, BannerOption};
use crate::db::repo::Repo;
use crate::ellucian::EllucianClient;
use crate::ellucian::models::{EllucianClass, InstructionalEvent};
use crate::error::AppError;
use crate::http::{HttpClient, RateLimiter};
use crate::moodle::models::{
    MoodleAnnouncement, MoodleAssignment, MoodleCourse, MoodleEvent, MoodleFile, MoodleGradeTotal,
    MoodleSection,
};
use crate::moodle::{MoodleClient, parser as moodle_parser};

/// What the page of an activity returns: submission state, text and
/// attachments. It is requested once and serves all three.
#[derive(Debug, Clone, Default)]
pub struct MoodleActivityPage {
    pub assignment: Option<MoodleAssignment>,
    pub description: String,
    pub files: Vec<MoodleFile>,
}

#[async_trait]
pub trait MoodleSource: Send + Sync {
    async fn health(&self) -> Result<bool, AppError>;
    async fn courses(&self) -> Result<Vec<MoodleCourse>, AppError>;
    async fn calendar(&self) -> Result<Vec<MoodleEvent>, AppError>;
    async fn sections(&self, course_id: i64) -> Result<Vec<MoodleSection>, AppError>;
    async fn activity(&self, url: &str) -> Result<MoodleActivityPage, AppError>;
    async fn announcements(&self, course_id: i64) -> Result<Vec<MoodleAnnouncement>, AppError>;
    async fn grade_total(&self, course_id: i64) -> Result<Option<MoodleGradeTotal>, AppError>;
    async fn download(&self, url: &str, dest: &std::path::Path) -> Result<PathBuf, AppError>;
}

#[async_trait]
pub trait EllucianSource: Send + Sync {
    async fn health(&self) -> Result<bool, AppError>;
    async fn classes(&self) -> Result<Vec<EllucianClass>, AppError>;
    async fn schedule(
        &self,
        classes: &[EllucianClass],
    ) -> Result<Vec<InstructionalEvent>, AppError>;
}

#[async_trait]
pub trait BannerSource: Send + Sync {
    async fn health(&self) -> Result<bool, AppError>;
    async fn periods(&self) -> Result<Vec<BannerOption>, AppError>;
    async fn levels(&self, term: &str) -> Result<Vec<BannerOption>, AppError>;
    async fn grades(&self, term: &str, level: &str) -> Result<Vec<BannerGrade>, AppError>;
}

pub struct SyncDeps {
    pub moodle: Arc<dyn MoodleSource>,
    pub ellucian: Arc<dyn EllucianSource>,
    pub banner: Arc<dyn BannerSource>,
    pub repo: Repo,
    pub limiter: Arc<RateLimiter>,
    pub emitter: Option<crate::sync::EventEmitter>,
}

pub struct LiveSources {
    session: Arc<SessionManager>,
    moodle: MoodleClient,
    ellucian: EllucianClient,
    banner: BannerClient,
}

impl LiveSources {
    pub fn new(http: HttpClient, session: Arc<SessionManager>) -> Self {
        let moodle = MoodleClient::new(http.clone(), Arc::clone(&session));
        let ellucian = EllucianClient::new(http.clone(), Arc::clone(&session));
        let banner = BannerClient::new(http.clone(), Arc::clone(&session));

        Self {
            session,
            moodle,
            ellucian,
            banner,
        }
    }
}

#[async_trait]
impl MoodleSource for LiveSources {
    async fn health(&self) -> Result<bool, AppError> {
        self.session.health(Source::Moodle).await
    }

    async fn courses(&self) -> Result<Vec<MoodleCourse>, AppError> {
        self.moodle.fetch_courses().await
    }

    async fn calendar(&self) -> Result<Vec<MoodleEvent>, AppError> {
        self.moodle.fetch_calendar().await
    }

    async fn sections(&self, course_id: i64) -> Result<Vec<MoodleSection>, AppError> {
        self.moodle.fetch_sections(course_id).await
    }

    async fn activity(&self, url: &str) -> Result<MoodleActivityPage, AppError> {
        let html = self.moodle.fetch_page(url).await?;
        let assignment = if url.contains("/mod/assign/") {
            moodle_parser::parse_assignment_html(&html).ok()
        } else {
            None
        };

        Ok(MoodleActivityPage {
            assignment,
            description: moodle_parser::parse_description(&html),
            files: moodle_parser::parse_attachments(&html),
        })
    }

    async fn announcements(&self, course_id: i64) -> Result<Vec<MoodleAnnouncement>, AppError> {
        self.moodle.fetch_announcements(course_id).await
    }

    async fn grade_total(&self, course_id: i64) -> Result<Option<MoodleGradeTotal>, AppError> {
        self.moodle.fetch_grade_total(course_id).await
    }

    async fn download(&self, url: &str, dest: &std::path::Path) -> Result<PathBuf, AppError> {
        self.moodle.download_file(url, dest).await
    }
}

#[async_trait]
impl EllucianSource for LiveSources {
    async fn health(&self) -> Result<bool, AppError> {
        self.session.health(Source::Ellucian).await
    }

    async fn classes(&self) -> Result<Vec<EllucianClass>, AppError> {
        self.ellucian.fetch_classes().await
    }

    async fn schedule(
        &self,
        classes: &[EllucianClass],
    ) -> Result<Vec<InstructionalEvent>, AppError> {
        self.ellucian.fetch_schedule_for(classes).await
    }
}

#[async_trait]
impl BannerSource for LiveSources {
    async fn health(&self) -> Result<bool, AppError> {
        self.session.health(Source::Banner).await
    }

    async fn periods(&self) -> Result<Vec<BannerOption>, AppError> {
        self.banner.terms().await
    }

    async fn levels(&self, term: &str) -> Result<Vec<BannerOption>, AppError> {
        self.banner.levels(term).await
    }

    async fn grades(&self, term: &str, level: &str) -> Result<Vec<BannerGrade>, AppError> {
        self.banner.courses(term, level, "-2").await
    }
}
