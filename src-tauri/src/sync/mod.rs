pub mod diff;
pub mod downloads;
pub mod orchestrator;
pub mod sources;

pub use crate::events::EventEmitter;
pub use contracts::{SyncRun, SyncStatus, SyncStatusKind};
pub use diff::{acknowledge, archive_stale, is_baseline, mark_new};
pub use downloads::{Downloads, safe_name};
pub use orchestrator::SyncOrchestrator;
pub use sources::{
    BannerSource, EllucianSource, LiveSources, MoodleActivityPage, MoodleSource, SyncDeps,
};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncTrigger {
    Startup,
    Manual,
    Scheduled,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "lowercase")]
pub enum SyncEvent {
    Progress(SyncStatus),
    Finished(SyncRun),
}
