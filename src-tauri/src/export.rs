use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};

use crate::db::models::{
    Activity, Announcement, Course, CourseGroup, FileItem, GradeItem, GradeTotal, ScheduleEvent,
    Section, SourceStatus,
};
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub version: u32,
    pub exported_at: i64,
    pub course_groups: Vec<CourseGroup>,
    pub courses: Vec<Course>,
    pub sections: Vec<Section>,
    pub activities: Vec<Activity>,
    pub files: Vec<FileItem>,
    pub announcements: Vec<Announcement>,
    pub grade_items: Vec<GradeItem>,
    pub grade_totals: Vec<GradeTotal>,
    pub schedule_events: Vec<ScheduleEvent>,
    pub settings: Vec<Setting>,
    pub source_status: Vec<SourceStatus>,
}

pub async fn snapshot(pool: &SqlitePool) -> Result<Snapshot, AppError> {
    let mut transaction = pool.begin().await.map_err(fail)?;

    let course_groups = rows(&mut transaction, "SELECT * FROM course_groups ORDER BY id").await?;
    let courses = rows(&mut transaction, "SELECT * FROM courses ORDER BY id").await?;
    let sections = rows(
        &mut transaction,
        "SELECT * FROM sections ORDER BY course_id, position",
    )
    .await?;
    let activities = rows(&mut transaction, "SELECT * FROM activities ORDER BY id").await?;
    let files = rows(&mut transaction, "SELECT * FROM files ORDER BY id").await?;
    let announcements = rows(&mut transaction, "SELECT * FROM announcements ORDER BY id").await?;
    let grade_items = rows(&mut transaction, "SELECT * FROM grade_items ORDER BY id").await?;
    let grade_totals = rows(
        &mut transaction,
        "SELECT * FROM grade_totals ORDER BY course_id, term",
    )
    .await?;
    let schedule_events = rows(
        &mut transaction,
        "SELECT * FROM schedule_events ORDER BY id",
    )
    .await?;
    let source_status = rows(
        &mut transaction,
        "SELECT * FROM source_status ORDER BY source",
    )
    .await?;

    let settings: Vec<Setting> =
        sqlx::query_as::<_, (String, String)>("SELECT key, value FROM settings ORDER BY key")
            .fetch_all(&mut *transaction)
            .await
            .map_err(fail)?
            .into_iter()
            .map(|(key, value)| Setting { key, value })
            .collect();

    let exported_at = Utc::now().timestamp();
    transaction.commit().await.map_err(fail)?;

    Ok(Snapshot {
        version: 1,
        exported_at,
        course_groups,
        courses,
        sections,
        activities,
        files,
        announcements,
        grade_items,
        grade_totals,
        schedule_events,
        settings,
        source_status,
    })
}

pub fn snapshot_json(snapshot: &Snapshot) -> Result<String, AppError> {
    serde_json::to_string_pretty(snapshot)
        .map_err(|error| AppError::Internal(format!("could not serialize the export: {error}")))
}

pub async fn write_snapshot(dir: &Path, pool: &SqlitePool) -> Result<PathBuf, AppError> {
    let snapshot = snapshot(pool).await?;
    let json = snapshot_json(&snapshot)?;

    tokio::fs::create_dir_all(dir).await.map_err(|error| {
        AppError::Internal(format!("could not create the export folder: {error}"))
    })?;

    let path = dir.join(format!(
        "rust-app-export-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S-%3f")
    ));
    tokio::fs::write(&path, json)
        .await
        .map_err(|error| AppError::Internal(format!("could not write the export: {error}")))?;

    Ok(path)
}

async fn rows<T>(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    sql: &str,
) -> Result<Vec<T>, AppError>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    sqlx::query_as::<_, T>(sql)
        .fetch_all(&mut **transaction)
        .await
        .map_err(fail)
}

fn fail(error: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("could not export the data: {error}"))
}
