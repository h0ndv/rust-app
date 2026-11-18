use serde::Serialize;
use sqlx::SqlitePool;

use crate::db::models::{
    Activity, Announcement, Course, CourseGroup, FileItem, GradeItem, GradeTotal, ScheduleEvent,
    Section, SourceStatus, SyncRun,
};
use crate::error::AppError;

const UPSERT_COURSE: &str = "INSERT INTO courses (
        id, group_id, source, moodle_id, banner_crn, shortname, fullname,
        section, term, teacher, moodle_url, banner_url, last_synced_at
     ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
     ON CONFLICT(id) DO UPDATE SET
        group_id = excluded.group_id,
        source = excluded.source,
        moodle_id = excluded.moodle_id,
        banner_crn = excluded.banner_crn,
        shortname = excluded.shortname,
        fullname = excluded.fullname,
        section = excluded.section,
        term = excluded.term,
        teacher = excluded.teacher,
        moodle_url = excluded.moodle_url,
        banner_url = excluded.banner_url,
        last_synced_at = COALESCE(excluded.last_synced_at, courses.last_synced_at)";

#[derive(Debug, Clone, Serialize)]
pub struct CourseSectionDto {
    pub id: String,
    pub fullname: String,
    pub shortname: Option<String>,
    pub section: Option<String>,
    pub term: Option<String>,
    pub teacher: Option<String>,
    pub source: String,
    pub banner_crn: Option<String>,
    pub next_due_at: Option<i64>,
    pub pending_tasks: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseGroupDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub sections: Vec<CourseSectionDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseDetailDto {
    pub group: CourseGroupDto,
    pub tasks: Vec<Activity>,
    pub resources: Vec<FileItem>,
    pub grades: Vec<GradeItem>,
    pub schedule: Vec<ScheduleEvent>,
}

#[derive(sqlx::FromRow)]
struct SectionRow {
    id: String,
    fullname: String,
    shortname: Option<String>,
    section: Option<String>,
    term: Option<String>,
    teacher: Option<String>,
    source: String,
    banner_crn: Option<String>,
    next_due_at: Option<i64>,
    pending_tasks: i64,
}

/// Repository over the pool.
#[derive(Clone)]
pub struct Repo {
    pool: SqlitePool,
}

impl Repo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Shared pool, for one-off queries from the sync.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn course_count(&self) -> Result<i64, AppError> {
        sqlx::query_scalar("SELECT count(*) FROM courses")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo contar cursos: {e}")))
    }

    pub async fn upsert_course(&self, course: &Course) -> Result<(), AppError> {
        sqlx::query(UPSERT_COURSE)
            .bind(&course.id)
            .bind(&course.group_id)
            .bind(&course.source)
            .bind(course.moodle_id)
            .bind(&course.banner_crn)
            .bind(&course.shortname)
            .bind(&course.fullname)
            .bind(&course.section)
            .bind(&course.term)
            .bind(&course.teacher)
            .bind(&course.moodle_url)
            .bind(&course.banner_url)
            .bind(&course.last_synced_at)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo guardar {}: {e}", course.id)))?;
        Ok(())
    }

    /// Groups and courses in a single transaction: the catalog is replaced or
    /// updated by id, never duplicated.
    pub async fn upsert_catalog(
        &self,
        groups: &[CourseGroup],
        courses: &[Course],
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo abrir la transacción: {e}")))?;

        for group in groups {
            sqlx::query(
                "INSERT INTO course_groups (id, code, name) VALUES (?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET code = excluded.code, name = excluded.name",
            )
            .bind(&group.id)
            .bind(&group.code)
            .bind(&group.name)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo guardar {}: {e}", group.code)))?;
        }

        for course in courses {
            sqlx::query(UPSERT_COURSE)
                .bind(&course.id)
                .bind(&course.group_id)
                .bind(&course.source)
                .bind(course.moodle_id)
                .bind(&course.banner_crn)
                .bind(&course.shortname)
                .bind(&course.fullname)
                .bind(&course.section)
                .bind(&course.term)
                .bind(&course.teacher)
                .bind(&course.moodle_url)
                .bind(&course.banner_url)
                .bind(&course.last_synced_at)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("no se pudo guardar {}: {e}", course.id))
                })?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo confirmar: {e}")))
    }

    pub async fn list_course_groups(&self) -> Result<Vec<CourseGroupDto>, AppError> {
        let groups: Vec<CourseGroup> = sqlx::query_as("SELECT * FROM course_groups ORDER BY code")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudieron leer los grupos: {e}")))?;

        let mut out = Vec::with_capacity(groups.len());
        for group in groups {
            let sections = self.sections_for(&group.id).await?;
            out.push(CourseGroupDto {
                id: group.id,
                code: group.code,
                name: group.name,
                sections,
            });
        }
        Ok(out)
    }

    async fn sections_for(&self, group_id: &str) -> Result<Vec<CourseSectionDto>, AppError> {
        let now = chrono::Utc::now().timestamp();

        let rows: Vec<SectionRow> = sqlx::query_as(
            "SELECT c.id, c.fullname, c.shortname, c.section, c.term, c.teacher, c.source,
                    c.banner_crn,
                    (SELECT MIN(a.due_at) FROM activities a
                      WHERE a.course_id = c.id AND a.archived_at IS NULL AND a.due_at > ?
                        AND COALESCE(a.submitted, 0) = 0 AND COALESCE(a.graded, 0) = 0)
                        AS next_due_at,
                    (SELECT COUNT(*) FROM activities a
                      WHERE a.course_id = c.id AND a.archived_at IS NULL
                        AND COALESCE(a.submitted, 0) = 0 AND COALESCE(a.graded, 0) = 0)
                        AS pending_tasks
             FROM courses c
             WHERE c.group_id = ?
             ORDER BY c.section",
        )
        .bind(now)
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudieron leer las secciones: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|row| CourseSectionDto {
                id: row.id,
                fullname: row.fullname,
                shortname: row.shortname,
                section: row.section,
                term: row.term,
                teacher: row.teacher,
                source: row.source,
                banner_crn: row.banner_crn,
                next_due_at: row.next_due_at,
                pending_tasks: row.pending_tasks,
            })
            .collect())
    }

    pub async fn get_course_detail(&self, group_id: &str) -> Result<CourseDetailDto, AppError> {
        let group: CourseGroup = sqlx::query_as("SELECT * FROM course_groups WHERE id = ?")
            .bind(group_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo leer el grupo: {e}")))?
            .ok_or(AppError::NotFound)?;

        let sections = self.sections_for(&group.id).await?;

        let tasks: Vec<Activity> = sqlx::query_as(
            "SELECT a.* FROM activities a
             JOIN courses c ON c.id = a.course_id
             WHERE c.group_id = ? AND a.archived_at IS NULL
             ORDER BY COALESCE(a.due_at, 0)",
        )
        .bind(&group.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudieron leer las tareas: {e}")))?;

        let resources: Vec<FileItem> = sqlx::query_as(
            "SELECT f.* FROM files f
             JOIN courses c ON c.id = f.course_id
             WHERE c.group_id = ? AND f.archived_at IS NULL
             ORDER BY f.name",
        )
        .bind(&group.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudieron leer los recursos: {e}")))?;

        let grades: Vec<GradeItem> = sqlx::query_as(
            "SELECT g.* FROM grade_items g
             JOIN courses c ON c.id = g.course_id
             WHERE c.group_id = ?
             ORDER BY g.name",
        )
        .bind(&group.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudieron leer las notas: {e}")))?;

        let schedule: Vec<ScheduleEvent> = sqlx::query_as(
            "SELECT s.* FROM schedule_events s
             JOIN courses c ON c.id = s.course_id
             WHERE c.group_id = ?
             ORDER BY s.day_of_week, s.start_time",
        )
        .bind(&group.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo leer el horario: {e}")))?;

        Ok(CourseDetailDto {
            group: CourseGroupDto {
                id: group.id,
                code: group.code,
                name: group.name,
                sections,
            },
            tasks,
            resources,
            grades,
            schedule,
        })
    }
}

impl Repo {
    pub async fn upsert_section(&self, section: &Section) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sections (id, course_id, name, position) VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                name = excluded.name,
                position = excluded.position",
        )
        .bind(&section.id)
        .bind(&section.course_id)
        .bind(&section.name)
        .bind(section.position)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::Internal(format!("no se pudo guardar la sección {}: {e}", section.id))
        })
        .map(|_| ())
    }

    pub async fn upsert_activity(&self, activity: &Activity) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO activities (
                id, course_id, section_id, cmid, name, modtype, url, due_at, open_at,
                cutoff_at, status, submitted, graded, grade, max_grade, description,
                content_hash, first_seen_at, acknowledged_at, archived_at, last_seen_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                section_id = excluded.section_id,
                cmid = excluded.cmid,
                name = excluded.name,
                modtype = excluded.modtype,
                url = excluded.url,
                due_at = excluded.due_at,
                open_at = excluded.open_at,
                cutoff_at = excluded.cutoff_at,
                status = excluded.status,
                submitted = excluded.submitted,
                graded = excluded.graded,
                grade = excluded.grade,
                max_grade = excluded.max_grade,
                description = excluded.description,
                content_hash = excluded.content_hash,
                first_seen_at = COALESCE(activities.first_seen_at, excluded.first_seen_at),
                acknowledged_at = COALESCE(activities.acknowledged_at, excluded.acknowledged_at),
                last_seen_at = excluded.last_seen_at",
        )
        .bind(&activity.id)
        .bind(&activity.course_id)
        .bind(&activity.section_id)
        .bind(activity.cmid)
        .bind(&activity.name)
        .bind(&activity.modtype)
        .bind(&activity.url)
        .bind(activity.due_at)
        .bind(activity.open_at)
        .bind(activity.cutoff_at)
        .bind(&activity.status)
        .bind(activity.submitted)
        .bind(activity.graded)
        .bind(activity.grade)
        .bind(activity.max_grade)
        .bind(&activity.description)
        .bind(&activity.content_hash)
        .bind(&activity.first_seen_at)
        .bind(&activity.acknowledged_at)
        .bind(&activity.archived_at)
        .bind(&activity.last_seen_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::Internal(format!(
                "no se pudo guardar la actividad {}: {e}",
                activity.id
            ))
        })
        .map(|_| ())
    }

    pub async fn upsert_file(&self, file: &FileItem) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO files (
                id, activity_id, course_id, name, url, mimetype, size, kind, sha256,
                local_path, first_seen_at, acknowledged_at, archived_at, last_seen_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                activity_id = excluded.activity_id,
                course_id = excluded.course_id,
                name = excluded.name,
                url = excluded.url,
                mimetype = excluded.mimetype,
                size = excluded.size,
                kind = excluded.kind,
                first_seen_at = COALESCE(files.first_seen_at, excluded.first_seen_at),
                acknowledged_at = COALESCE(files.acknowledged_at, excluded.acknowledged_at),
                last_seen_at = excluded.last_seen_at",
        )
        .bind(&file.id)
        .bind(&file.activity_id)
        .bind(&file.course_id)
        .bind(&file.name)
        .bind(&file.url)
        .bind(&file.mimetype)
        .bind(file.size)
        .bind(&file.kind)
        .bind(&file.sha256)
        .bind(&file.local_path)
        .bind(&file.first_seen_at)
        .bind(&file.acknowledged_at)
        .bind(&file.archived_at)
        .bind(&file.last_seen_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo guardar el archivo {}: {e}", file.id)))
        .map(|_| ())
    }

    pub async fn upsert_grade_item(&self, item: &GradeItem) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO grade_items (
                id, course_id, term, name, category, grade, max_grade, weight,
                contribution, source, raw_json
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                term = excluded.term,
                name = excluded.name,
                category = excluded.category,
                grade = excluded.grade,
                max_grade = excluded.max_grade,
                weight = excluded.weight,
                contribution = excluded.contribution,
                source = excluded.source,
                raw_json = excluded.raw_json",
        )
        .bind(&item.id)
        .bind(&item.course_id)
        .bind(&item.term)
        .bind(&item.name)
        .bind(&item.category)
        .bind(item.grade)
        .bind(item.max_grade)
        .bind(item.weight)
        .bind(item.contribution)
        .bind(&item.source)
        .bind(&item.raw_json)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo guardar la nota {}: {e}", item.id)))
        .map(|_| ())
    }

    pub async fn upsert_grade_total(&self, total: &GradeTotal) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO grade_totals (course_id, term, total, max_total, source, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(course_id, term, source) DO UPDATE SET
                total = excluded.total,
                max_total = excluded.max_total,
                updated_at = excluded.updated_at",
        )
        .bind(&total.course_id)
        .bind(&total.term)
        .bind(total.total)
        .bind(total.max_total)
        .bind(&total.source)
        .bind(&total.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo guardar el total: {e}")))
        .map(|_| ())
    }

    pub async fn upsert_schedule_event(&self, event: &ScheduleEvent) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO schedule_events (
                id, course_id, title, day_of_week, start_time, end_time, room,
                building, instructor, term, raw_json
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                title = excluded.title,
                day_of_week = excluded.day_of_week,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                room = excluded.room,
                building = excluded.building,
                instructor = excluded.instructor,
                term = excluded.term,
                raw_json = excluded.raw_json",
        )
        .bind(&event.id)
        .bind(&event.course_id)
        .bind(&event.title)
        .bind(event.day_of_week)
        .bind(&event.start_time)
        .bind(&event.end_time)
        .bind(&event.room)
        .bind(&event.building)
        .bind(&event.instructor)
        .bind(&event.term)
        .bind(&event.raw_json)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo guardar el bloque {}: {e}", event.id)))
        .map(|_| ())
    }

    pub async fn upsert_announcement(&self, announcement: &Announcement) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO announcements (
                id, course_id, title, body, url, published_at, first_seen_at, acknowledged_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                title = excluded.title,
                body = excluded.body,
                url = excluded.url,
                published_at = excluded.published_at,
                first_seen_at = COALESCE(announcements.first_seen_at, excluded.first_seen_at),
                acknowledged_at = COALESCE(announcements.acknowledged_at, excluded.acknowledged_at)",
        )
        .bind(&announcement.id)
        .bind(&announcement.course_id)
        .bind(&announcement.title)
        .bind(&announcement.body)
        .bind(&announcement.url)
        .bind(announcement.published_at)
        .bind(&announcement.first_seen_at)
        .bind(&announcement.acknowledged_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo guardar el anuncio {}: {e}", announcement.id)))
        .map(|_| ())
    }

    pub async fn upsert_source_status(&self, status: &SourceStatus) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO source_status (source, state, last_checked_at, last_error)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(source) DO UPDATE SET
                state = excluded.state,
                last_checked_at = excluded.last_checked_at,
                last_error = excluded.last_error",
        )
        .bind(&status.source)
        .bind(&status.state)
        .bind(&status.last_checked_at)
        .bind(&status.last_error)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo guardar el estado: {e}")))
        .map(|_| ())
    }

    pub async fn insert_sync_run(&self, run: &SyncRun) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sync_runs (id, started_at, finished_at, status, requests, errors_json)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&run.id)
        .bind(&run.started_at)
        .bind(&run.finished_at)
        .bind(&run.status)
        .bind(run.requests)
        .bind(&run.errors_json)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo abrir la corrida: {e}")))
        .map(|_| ())
    }

    pub async fn finish_sync_run(
        &self,
        id: &str,
        status: &str,
        requests: i64,
        errors_json: Option<&str>,
        finished_at: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE sync_runs SET status = ?, requests = ?, errors_json = ?, finished_at = ?
             WHERE id = ?",
        )
        .bind(status)
        .bind(requests)
        .bind(errors_json)
        .bind(finished_at)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo cerrar la corrida: {e}")))
        .map(|_| ())
    }

    /// Stored hash and submission state, to avoid re-requesting the page of an
    /// activity that did not change.
    pub async fn activity_state(
        &self,
        id: &str,
    ) -> Result<Option<(Option<String>, Option<bool>)>, AppError> {
        sqlx::query_as("SELECT content_hash, submitted FROM activities WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo leer la actividad: {e}")))
    }

    pub async fn set_course_synced(&self, course_id: &str, at: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE courses SET last_synced_at = ? WHERE id = ?")
            .bind(at)
            .bind(course_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo marcar el curso: {e}")))
            .map(|_| ())
    }

    pub async fn file_for_download(
        &self,
        id: &str,
    ) -> Result<Option<(String, String, Option<String>)>, AppError> {
        sqlx::query_as("SELECT url, name, mimetype FROM files WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo leer el archivo: {e}")))
    }

    pub async fn mark_downloaded(&self, id: &str, path: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE files SET local_path = ? WHERE id = ?")
            .bind(path)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo guardar la ruta: {e}")))
            .map(|_| ())
    }
}

impl Repo {
    pub async fn file_exists(&self, id: &str) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM files WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo leer el archivo: {e}")))?;
        Ok(count > 0)
    }

    pub async fn announcement_exists(&self, id: &str) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM announcements WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo leer el anuncio: {e}")))?;
        Ok(count > 0)
    }
}

impl Repo {
    /// Id of the row that already represents that CRN, preferring the Moodle one.
    pub async fn course_id_by_crn(&self, crn: &str) -> Result<Option<String>, AppError> {
        sqlx::query_scalar(
            "SELECT id FROM courses WHERE banner_crn = ?
             ORDER BY (moodle_id IS NULL) LIMIT 1",
        )
        .bind(crn)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("no se pudo buscar el CRN: {e}")))
    }
}

impl Repo {
    /// Closes the runs left in `running` by an abrupt shutdown.
    pub async fn finish_stale_runs(&self, at: &str) -> Result<u64, AppError> {
        sqlx::query(
            "UPDATE sync_runs SET status = 'failed', finished_at = ?, errors_json = ?
             WHERE status = 'running'",
        )
        .bind(at)
        .bind(r#"["corrida interrumpida"]"#)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(|e| AppError::Internal(format!("no se pudieron cerrar corridas: {e}")))
    }
}

impl Repo {
    /// Keeps the search index up to date. FTS5 has no upsert: the previous row
    /// is deleted and the new one is inserted.
    pub async fn index_entity(
        &self,
        kind: &str,
        id: &str,
        title: &str,
        body: &str,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM search_index WHERE kind = ? AND entity_id = ?")
            .bind(kind)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo limpiar el índice: {e}")))?;

        sqlx::query("INSERT INTO search_index (kind, entity_id, title, body) VALUES (?, ?, ?, ?)")
            .bind(kind)
            .bind(id)
            .bind(title)
            .bind(body)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo indexar: {e}")))
            .map(|_| ())
    }
}

impl Repo {
    /// Keeps index statistics fresh and trims the WAL after a sync. Safe to
    /// call on an in-memory database; both statements are no-ops there.
    pub async fn optimize(&self) -> Result<(), AppError> {
        sqlx::query("PRAGMA optimize")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo optimizar la base: {e}")))?;

        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("no se pudo recortar el WAL: {e}")))?;

        Ok(())
    }
}
