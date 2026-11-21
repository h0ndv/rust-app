pub mod diff;
pub mod downloads;
pub mod orchestrator;
pub mod sources;

pub use crate::events::EventEmitter;
pub use diff::{acknowledge, archive_stale, is_baseline, mark_new};
pub use downloads::{Downloads, safe_name};
pub use orchestrator::SyncOrchestrator;
pub use sources::{
    BannerSource, EllucianSource, LiveSources, MoodleActivityPage, MoodleSource, SyncDeps,
};

use serde::Serialize;

use crate::auth::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncTrigger {
    Startup,
    Manual,
    Scheduled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatusKind {
    Running,
    Ok,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncRun {
    pub id: String,
    pub status: SyncStatusKind,
    pub requests: u32,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncStatus {
    pub state: SyncStatusKind,
    pub source: Option<Source>,
    pub progress: u8,
    pub requests: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "lowercase")]
pub enum SyncEvent {
    Progress(SyncStatus),
    Finished(SyncRun),
}
