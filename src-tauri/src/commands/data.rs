use chrono::{Datelike, Duration, Local, Utc};
use sqlx::FromRow;
use tauri::State;

use crate::db::repo::CourseGroupDto;
use crate::error::AppError;
use crate::state::AppState;

use super::Cmd;
use super::dto::{
    DashboardDto, GradeRowDto, GradeTotalDto, GradesDto, NewsItemDto, NewsKind, ResourceDto,
    ResourceFilter, ScheduleEventDto, SearchHitDto, SearchResultsDto, SettingsDto, TaskDto,
    TaskFilter,
};

fn fail(error: impl std::fmt::Display) -> AppError {
    AppError::Internal(error.to_string())
}

pub fn normalize_language(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    let valid = !value.is_empty()
        && value.len() <= 35
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-');

    if valid { value } else { "en".to_string() }
}

async fn upsert_setting(pool: &sqlx::SqlitePool, key: &str, value: &str) -> Cmd<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(fail)?;

    Ok(())
}

pub fn normalize_status(value: Option<&str>) -> Option<String> {
    value.map(|value| {
        match value {
            "atrasada" => "overdue",
            "pendiente" => "pending",
            "entregada" => "submitted",
            "calificada" => "graded",
            "sin_datos" => "no_data",
            "por_confirmar" => "unconfirmed",
            "informativa" => "informational",
            other => other,
        }
        .to_string()
    })
}

pub fn normalize_kind(value: &str) -> String {
    match value {
        "archivo" => "file",
        other => other,
    }
    .to_string()
}

pub fn normalize_schedule_title(value: &str) -> String {
    if value == "Clase" {
        return String::new();
    }
    value.to_string()
}

#[derive(FromRow)]
struct TaskRow {
    id: String,
    course_id: String,
    course_fullname: String,
    name: String,
    due_at: Option<i64>,
    status: Option<String>,
    submitted: Option<bool>,
    grade: Option<f64>,
    url: Option<String>,
    acknowledged_at: Option<String>,
}

#[derive(FromRow)]
struct ResourceRow {
    id: String,
    course_id: Option<String>,
    course_fullname: Option<String>,
    name: String,
    kind: String,
    mimetype: Option<String>,
    size: Option<i64>,
    url: String,
    acknowledged_at: Option<String>,
    local_path: Option<String>,
}

#[derive(FromRow)]
struct ScheduleRow {
    id: String,
    course_title: Option<String>,
    title: String,
    day_of_week: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    room: Option<String>,
    building: Option<String>,
    instructor: Option<String>,
}

#[derive(FromRow)]
struct GradeRow {
    course_fullname: String,
    name: String,
    grade: Option<f64>,
    max_grade: Option<f64>,
    source: String,
}

#[derive(FromRow)]
struct TotalRow {
    course_fullname: String,
    total: Option<f64>,
    max_total: Option<f64>,
    source: String,
}

#[derive(FromRow)]
struct NewsRow {
    kind: String,
    id: String,
    course_title: Option<String>,
    group_id: Option<String>,
    title: String,
    body: String,
    published_at: Option<i64>,
}

impl From<TaskRow> for TaskDto {
    fn from(row: TaskRow) -> Self {
        Self {
            id: row.id,
            course_id: row.course_id,
            course_name: row.course_fullname,
            title: row.name,
            due_at: row.due_at,
            status: normalize_status(row.status.as_deref()),
            submitted: row.submitted.unwrap_or(false),
            grade: row.grade,
            url: row.url,
            is_new: row.acknowledged_at.is_none(),
        }
    }
}

impl From<ResourceRow> for ResourceDto {
    fn from(row: ResourceRow) -> Self {
        Self {
            id: row.id,
            course_id: row.course_id,
            course: row.course_fullname,
            title: row.name,
            kind: normalize_kind(&row.kind),
            mimetype: row.mimetype,
            size: row.size,
            url: row.url,
            is_new: row.acknowledged_at.is_none(),
            downloaded: row.local_path.is_some(),
        }
    }
}

impl From<ScheduleRow> for ScheduleEventDto {
    fn from(row: ScheduleRow) -> Self {
        Self {
            id: row.id,
            course: row.course_title,
            title: row.title,
            day: row.day_of_week,
            start: row.start_time,
            end: row.end_time,
            room: row.room,
            building: row.building,
            instructor: row.instructor,
        }
    }
}

async fn load_tasks(state: &AppState) -> Cmd<Vec<TaskDto>> {
    let rows: Vec<TaskRow> = sqlx::query_as(
        "SELECT a.id, a.course_id, c.fullname AS course_fullname, a.name, a.due_at,
                a.status, a.submitted, a.grade, a.url, a.acknowledged_at
         FROM activities a
         JOIN courses c ON c.id = a.course_id
         WHERE a.archived_at IS NULL AND a.modtype IN ('assign', 'quiz')
         ORDER BY COALESCE(a.due_at, 0)",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    Ok(rows.into_iter().map(TaskDto::from).collect())
}

async fn load_resources(state: &AppState) -> Cmd<Vec<ResourceDto>> {
    let rows: Vec<ResourceRow> = sqlx::query_as(
        "SELECT f.id, f.course_id, c.fullname AS course_fullname, f.name, f.kind,
                f.mimetype, f.size, f.url, f.acknowledged_at, f.local_path
         FROM files f
         LEFT JOIN courses c ON c.id = f.course_id
         WHERE f.archived_at IS NULL
         ORDER BY f.name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    Ok(rows.into_iter().map(ResourceDto::from).collect())
}

async fn load_schedule(state: &AppState) -> Cmd<Vec<ScheduleEventDto>> {
    let rows: Vec<ScheduleRow> = sqlx::query_as(
        "SELECT s.id, c.fullname AS course_title, s.title, s.day_of_week, s.start_time,
                s.end_time, s.room, s.building, s.instructor
         FROM schedule_events s
         LEFT JOIN courses c ON c.id = s.course_id
         ORDER BY s.day_of_week, s.start_time",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    Ok(rows.into_iter().map(ScheduleEventDto::from).collect())
}

/// Orders the week from `today` (1 = Monday) and wraps into the next one, so a
/// Friday still sees Monday's class ahead.
fn next_classes(mut schedule: Vec<ScheduleEventDto>, today: i64) -> Vec<ScheduleEventDto> {
    schedule.retain(|event| event.day.is_some_and(|day| (1..=7).contains(&day)));

    let offset = |event: &ScheduleEventDto| {
        let day = event.day.unwrap_or(today);
        (day - today).rem_euclid(7)
    };
    schedule.sort_by(|left, right| {
        offset(left)
            .cmp(&offset(right))
            .then_with(|| left.start.cmp(&right.start))
    });

    schedule
}

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Cmd<DashboardDto> {
    let now = Utc::now().timestamp();
    let soon = now + Duration::days(7).num_seconds();

    let tasks = load_tasks(&state).await?;
    let overdue: Vec<TaskDto> = tasks
        .iter()
        .filter(|task| task.status.as_deref() == Some("overdue"))
        .cloned()
        .collect();
    let due_soon: Vec<TaskDto> = tasks
        .iter()
        .filter(|task| !task.submitted && task.due_at.is_some_and(|due| due >= now && due <= soon))
        .cloned()
        .collect();

    let schedule = load_schedule(&state).await?;
    let today = Local::now().weekday().number_from_monday() as i64;
    let upcoming: Vec<ScheduleEventDto> =
        next_classes(schedule, today).into_iter().take(5).collect();

    let new_resources: Vec<ResourceDto> = load_resources(&state)
        .await?
        .into_iter()
        .filter(|resource| resource.is_new)
        .take(5)
        .collect();

    Ok(DashboardDto {
        overdue,
        due_soon,
        next_classes: upcoming,
        new_resources,
    })
}

#[tauri::command]
pub async fn has_local_data(state: State<'_, AppState>) -> Cmd<bool> {
    Ok(state.repo.course_count().await? > 0)
}

#[tauri::command]
pub async fn list_courses(state: State<'_, AppState>) -> Cmd<Vec<CourseGroupDto>> {
    state.repo.list_course_groups().await
}

#[tauri::command]
pub async fn get_course(
    state: State<'_, AppState>,
    group_id: String,
) -> Cmd<crate::db::repo::CourseDetailDto> {
    let mut detail = state.repo.get_course_detail(&group_id).await?;

    for task in &mut detail.tasks {
        task.status = normalize_status(task.status.as_deref());
    }
    for resource in &mut detail.resources {
        resource.kind = normalize_kind(&resource.kind);
    }
    for block in &mut detail.schedule {
        block.title = normalize_schedule_title(&block.title);
    }

    Ok(detail)
}

#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>, filter: TaskFilter) -> Cmd<Vec<TaskDto>> {
    let tasks = load_tasks(&state).await?;

    Ok(tasks
        .into_iter()
        .filter(|task| match filter {
            TaskFilter::All => true,
            TaskFilter::Overdue => task.status.as_deref() == Some("overdue"),
            TaskFilter::Pending => task.status.as_deref() == Some("pending"),
            TaskFilter::Submitted => task.submitted,
            TaskFilter::Graded => task.grade.is_some(),
            TaskFilter::NoData => task.status.as_deref() == Some("no_data"),
            TaskFilter::Unconfirmed => task.status.as_deref() == Some("unconfirmed"),
            TaskFilter::Informational => task.status.as_deref() == Some("informational"),
        })
        .collect())
}

#[tauri::command]
pub async fn list_resources(
    state: State<'_, AppState>,
    filter: ResourceFilter,
) -> Cmd<Vec<ResourceDto>> {
    let resources = load_resources(&state).await?;

    Ok(resources
        .into_iter()
        .filter(|resource| match filter {
            ResourceFilter::All => true,
            ResourceFilter::New => resource.is_new,
            ResourceFilter::Downloaded => resource.downloaded,
        })
        .collect())
}

#[tauri::command]
pub async fn list_grades(state: State<'_, AppState>, term: Option<String>) -> Cmd<GradesDto> {
    let terms: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT term FROM grade_items ORDER BY term DESC")
            .fetch_all(&state.pool)
            .await
            .map_err(fail)?;

    let selected = term
        .or_else(|| terms.first().cloned())
        .unwrap_or_else(|| "202620".to_string());

    let banner: Vec<GradeRow> = sqlx::query_as(
        "SELECT c.fullname AS course_fullname, g.name, g.grade, g.max_grade, g.source
         FROM grade_items g
         JOIN courses c ON c.id = g.course_id
         WHERE g.source = 'banner' AND g.term = ?
         ORDER BY c.fullname",
    )
    .bind(&selected)
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    let totals: Vec<TotalRow> = sqlx::query_as(
        "SELECT c.fullname AS course_fullname, t.total, t.max_total, t.source
         FROM grade_totals t
         JOIN courses c ON c.id = t.course_id
         WHERE t.source = 'moodle'
         ORDER BY c.fullname",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    Ok(GradesDto {
        term: selected,
        terms,
        banner: banner
            .into_iter()
            .map(|row| GradeRowDto {
                course: row.course_fullname,
                name: row.name,
                grade: row.grade,
                max_grade: row.max_grade,
                source: row.source,
            })
            .collect(),
        moodle_totals: totals
            .into_iter()
            .map(|row| GradeTotalDto {
                course: row.course_fullname,
                total: row.total,
                max_total: row.max_total,
                source: row.source,
            })
            .collect(),
        gpa: None,
    })
}

#[tauri::command]
pub async fn get_schedule(state: State<'_, AppState>) -> Cmd<Vec<ScheduleEventDto>> {
    let mut events = load_schedule(&state).await?;
    for event in &mut events {
        event.title = normalize_schedule_title(&event.title);
    }
    Ok(events)
}

#[tauri::command]
pub async fn list_news(state: State<'_, AppState>) -> Cmd<Vec<NewsItemDto>> {
    news_items(&state.pool).await
}

pub async fn news_items(pool: &sqlx::SqlitePool) -> Cmd<Vec<NewsItemDto>> {
    let rows: Vec<NewsRow> = sqlx::query_as(
        "SELECT 'announcement' AS kind, a.id, c.fullname AS course_title, c.group_id,
                a.title, a.body, a.published_at
         FROM announcements a
         LEFT JOIN courses c ON c.id = a.course_id
         WHERE a.acknowledged_at IS NULL
         UNION ALL
         SELECT 'file', f.id, c.fullname, c.group_id, f.name, '', NULL
         FROM files f
         LEFT JOIN courses c ON c.id = f.course_id
         WHERE f.acknowledged_at IS NULL AND f.archived_at IS NULL
         UNION ALL
         SELECT 'activity', a.id, c.fullname, c.group_id, a.name, a.description, a.due_at
         FROM activities a
         LEFT JOIN courses c ON c.id = a.course_id
         WHERE a.acknowledged_at IS NULL AND a.archived_at IS NULL
           AND a.modtype IN ('assign', 'quiz')
         ORDER BY published_at DESC
         LIMIT 100",
    )
    .fetch_all(pool)
    .await
    .map_err(fail)?;

    Ok(rows
        .into_iter()
        .map(|row| NewsItemDto {
            id: row.id,
            course: row.course_title,
            group_id: row.group_id,
            title: row.title,
            body: row.body,
            published_at: row.published_at,
            kind: row.kind,
        })
        .collect())
}

#[tauri::command]
pub async fn acknowledge(state: State<'_, AppState>, kind: NewsKind, id: String) -> Cmd<()> {
    crate::sync::acknowledge(&state.pool, kind.as_str(), &id).await
}

#[derive(FromRow)]
struct SearchRow {
    kind: String,
    entity_id: String,
    title: String,
    body: String,
    group_id: Option<String>,
}

pub async fn search_hits(pool: &sqlx::SqlitePool, cleaned: &str) -> Cmd<Vec<SearchHitDto>> {
    let rows: Vec<SearchRow> = sqlx::query_as(
        "SELECT search_index.kind, search_index.entity_id,
                search_index.title, search_index.body, c.group_id AS group_id
         FROM search_index
         LEFT JOIN activities a
           ON search_index.kind = 'activity' AND a.id = search_index.entity_id
         LEFT JOIN files f
           ON search_index.kind = 'file' AND f.id = search_index.entity_id
         LEFT JOIN announcements an
           ON search_index.kind = 'announcement' AND an.id = search_index.entity_id
         LEFT JOIN courses c
           ON c.id = COALESCE(a.course_id, f.course_id, an.course_id)
         WHERE search_index MATCH ?
         LIMIT 50",
    )
    .bind(format!("\"{cleaned}\""))
    .fetch_all(pool)
    .await
    .map_err(fail)?;

    Ok(rows
        .into_iter()
        .map(|row| SearchHitDto {
            kind: row.kind,
            id: row.entity_id,
            title: row.title,
            body: row.body,
            group_id: row.group_id,
        })
        .collect())
}

#[tauri::command]
pub async fn search(state: State<'_, AppState>, query: String) -> Cmd<SearchResultsDto> {
    let cleaned: String = query
        .chars()
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .collect();
    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return Ok(SearchResultsDto {
            query,
            total: 0,
            items: Vec::new(),
        });
    }

    let items = search_hits(&state.pool, cleaned).await?;

    Ok(SearchResultsDto {
        query,
        total: items.len(),
        items,
    })
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Cmd<SettingsDto> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM settings")
        .fetch_all(&state.pool)
        .await
        .map_err(fail)?;

    let mut settings = SettingsDto::default();
    for (key, value) in rows {
        match key.as_str() {
            "sync_interval_minutes" => settings.sync_interval_minutes = value.parse().unwrap_or(30),
            "downloads_dir" => settings.downloads_dir = value,
            "theme" => settings.theme = value,
            "notifications" => settings.notifications = value == "true",
            "language" => settings.language = normalize_language(&value),
            _ => {}
        }
    }

    settings.language = normalize_language(&settings.language);

    if settings.downloads_dir.is_empty() {
        settings.downloads_dir = state.downloads_fallback.display().to_string();
    }

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: SettingsDto) -> Cmd<()> {
    let language = normalize_language(&settings.language);
    let pairs = [
        (
            "sync_interval_minutes",
            settings.sync_interval_minutes.to_string(),
        ),
        ("downloads_dir", settings.downloads_dir),
        ("theme", settings.theme),
        ("notifications", settings.notifications.to_string()),
        ("language", language),
    ];

    for (key, value) in pairs {
        upsert_setting(&state.pool, key, &value).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_language(state: State<'_, AppState>, language: String) -> Cmd<()> {
    let language = normalize_language(&language);
    upsert_setting(&state.pool, "language", &language).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_language_tags() {
        assert_eq!(normalize_language("en"), "en");
        assert_eq!(normalize_language("es"), "es");
        assert_eq!(normalize_language("ES"), "es");
        assert_eq!(normalize_language("fr"), "fr");
        assert_eq!(normalize_language("pt-BR"), "pt-br");
        assert_eq!(normalize_language("EN"), "en");
        assert_eq!(normalize_language(""), "en");
        assert_eq!(normalize_language("not a language"), "en");
    }

    #[test]
    fn normalizes_legacy_status_values() {
        assert_eq!(
            normalize_status(Some("atrasada")).as_deref(),
            Some("overdue")
        );
        assert_eq!(
            normalize_status(Some("pendiente")).as_deref(),
            Some("pending")
        );
        assert_eq!(
            normalize_status(Some("overdue")).as_deref(),
            Some("overdue")
        );
        assert_eq!(normalize_status(None), None);
    }

    #[test]
    fn normalizes_legacy_file_kind() {
        assert_eq!(normalize_kind("archivo"), "file");
        assert_eq!(normalize_kind("file"), "file");
    }

    #[test]
    fn normalizes_legacy_schedule_title() {
        assert_eq!(normalize_schedule_title("Clase"), "");
        assert_eq!(
            normalize_schedule_title("Minería de Datos"),
            "Minería de Datos"
        );
    }

    fn class(day: i64, start: &str) -> ScheduleEventDto {
        ScheduleEventDto {
            title: format!("clase {day} {start}"),
            day: Some(day),
            start: Some(start.to_string()),
            ..Default::default()
        }
    }

    fn days(schedule: &[ScheduleEventDto]) -> Vec<Option<i64>> {
        schedule.iter().map(|event| event.day).collect()
    }

    #[test]
    fn wraps_next_classes_into_the_next_week() {
        let schedule = vec![class(1, "08:00"), class(3, "10:00"), class(5, "09:00")];

        let ordered = next_classes(schedule, 5);

        assert_eq!(days(&ordered), vec![Some(5), Some(1), Some(3)]);
    }

    #[test]
    fn keeps_classes_of_the_current_day_in_time_order() {
        let schedule = vec![class(2, "12:00"), class(2, "08:00"), class(4, "09:00")];

        let ordered = next_classes(schedule, 2);

        let starts: Vec<Option<&str>> =
            ordered.iter().map(|event| event.start.as_deref()).collect();
        assert_eq!(starts, vec![Some("08:00"), Some("12:00"), Some("09:00")]);
    }

    #[test]
    fn drops_classes_without_a_weekday() {
        let schedule = vec![ScheduleEventDto::default(), class(2, "08:00")];

        let ordered = next_classes(schedule, 1);

        assert_eq!(days(&ordered), vec![Some(2)]);
    }

    #[tokio::test]
    async fn upsert_setting_writes_only_the_requested_key() {
        let pool = crate::db::connect(std::path::Path::new(":memory:"))
            .await
            .unwrap();
        crate::db::migrate(&pool).await.unwrap();

        upsert_setting(&pool, "language", "es").await.unwrap();
        upsert_setting(&pool, "theme", "dark").await.unwrap();

        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
                .fetch_all(&pool)
                .await
                .unwrap();

        assert_eq!(
            rows,
            vec![
                ("language".to_string(), "es".to_string()),
                ("theme".to_string(), "dark".to_string()),
            ]
        );
    }
}
