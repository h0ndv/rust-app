use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;

use crate::error::AppError;

/// A course is a baseline when it has no activities yet: everything that
/// appears in that run starts out already seen.
pub async fn is_baseline(pool: &SqlitePool, course_id: &str) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM activities WHERE course_id = ?")
        .bind(course_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo contar actividades: {e}")))?;
    Ok(count == 0)
}

pub async fn mark_new(pool: &SqlitePool, kind: &str, id: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO events (id, kind, entity_id, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(format!("{kind}:{id}"))
    .bind(kind)
    .bind(id)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(format!("no se pudo registrar la novedad: {e}")))
    .map(|_| ())
}

/// Marks an item as seen by the student.
pub async fn acknowledge(pool: &SqlitePool, kind: &str, id: &str) -> Result<(), AppError> {
    let table = match kind {
        "activity" => "activities",
        "file" => "files",
        "announcement" => "announcements",
        other => return Err(AppError::Internal(format!("tipo desconocido: {other}"))),
    };

    let now = Utc::now().to_rfc3339();
    sqlx::query(&format!(
        "UPDATE {table} SET acknowledged_at = ? WHERE id = ?"
    ))
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(format!("no se pudo reconocer el elemento: {e}")))?;

    sqlx::query("UPDATE events SET acknowledged_at = ? WHERE entity_id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo reconocer la novedad: {e}")))?;

    Ok(())
}

/// Archives what has not been seen for 7 days, only from courses that synced
/// well in the last hour. A failed course archives nothing.
pub async fn archive_stale(pool: &SqlitePool, now: DateTime<Utc>) -> Result<u64, AppError> {
    let cutoff = (now - Duration::days(7)).to_rfc3339();
    let synced_after = (now - Duration::hours(1)).to_rfc3339();
    let stamp = now.to_rfc3339();

    let activities = sqlx::query(
        "UPDATE activities SET archived_at = ?
         WHERE archived_at IS NULL
           AND (last_seen_at IS NULL OR last_seen_at < ?)
           AND course_id IN (
                SELECT id FROM courses
                WHERE last_synced_at IS NOT NULL AND last_synced_at >= ?
           )",
    )
    .bind(&stamp)
    .bind(&cutoff)
    .bind(&synced_after)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(format!("no se pudieron archivar tareas: {e}")))?
    .rows_affected();

    let files = sqlx::query(
        "UPDATE files SET archived_at = ?
         WHERE archived_at IS NULL
           AND (last_seen_at IS NULL OR last_seen_at < ?)
           AND course_id IN (
                SELECT id FROM courses
                WHERE last_synced_at IS NOT NULL AND last_synced_at >= ?
           )",
    )
    .bind(&stamp)
    .bind(&cutoff)
    .bind(&synced_after)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(format!("no se pudieron archivar recursos: {e}")))?
    .rows_affected();

    Ok(activities + files)
}
