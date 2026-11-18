use chrono::{Duration, Utc};
use sqlx::FromRow;
use tauri::State;

use crate::db::repo::CourseGroupDto;
use crate::state::AppState;

use super::dto::{
    DashboardDto, GradeRowDto, GradeTotalDto, GradesDto, NewsItemDto, ResourceDto, ResourceFilter,
    ScheduleEventDto, SearchHitDto, SearchResultsDto, SettingsDto, TaskDto, TaskFilter,
};

type Cmd<T> = Result<T, String>;

fn fail(error: impl std::fmt::Display) -> String {
    error.to_string()
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
}

#[derive(FromRow)]
struct ResourceRow {
    id: String,
    course_id: Option<String>,
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
    id: String,
    course_title: Option<String>,
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
            status: row.status,
            submitted: row.submitted.unwrap_or(false),
            grade: row.grade,
            url: row.url,
        }
    }
}

impl From<ResourceRow> for ResourceDto {
    fn from(row: ResourceRow) -> Self {
        Self {
            id: row.id,
            course_id: row.course_id,
            title: row.name,
            kind: row.kind,
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
                a.status, a.submitted, a.grade, a.url
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
        "SELECT id, course_id, name, kind, mimetype, size, url, acknowledged_at, local_path
         FROM files
         WHERE archived_at IS NULL
         ORDER BY name",
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

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Cmd<DashboardDto> {
    let now = Utc::now().timestamp();
    let soon = now + Duration::days(7).num_seconds();

    let tasks = load_tasks(&state).await?;
    let overdue: Vec<TaskDto> = tasks
        .iter()
        .filter(|task| task.status.as_deref() == Some("atrasada"))
        .cloned()
        .collect();
    let due_soon: Vec<TaskDto> = tasks
        .iter()
        .filter(|task| !task.submitted && task.due_at.is_some_and(|due| due >= now && due <= soon))
        .cloned()
        .collect();

    let schedule = load_schedule(&state).await?;
    let today = Utc::now()
        .format("%u")
        .to_string()
        .parse::<i64>()
        .unwrap_or(1);
    let next_classes: Vec<ScheduleEventDto> = schedule
        .into_iter()
        .filter(|event| event.day.is_some_and(|day| day >= today))
        .take(5)
        .collect();

    let new_resources: Vec<ResourceDto> = load_resources(&state)
        .await?
        .into_iter()
        .filter(|resource| resource.is_new)
        .take(5)
        .collect();

    Ok(DashboardDto {
        overdue,
        due_soon,
        next_classes,
        new_resources,
    })
}

#[tauri::command]
pub async fn list_courses(state: State<'_, AppState>) -> Cmd<Vec<CourseGroupDto>> {
    state.repo.list_course_groups().await.map_err(fail)
}

#[tauri::command]
pub async fn get_course(
    state: State<'_, AppState>,
    group_id: String,
) -> Cmd<crate::db::repo::CourseDetailDto> {
    state.repo.get_course_detail(&group_id).await.map_err(fail)
}

#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>, filter: TaskFilter) -> Cmd<Vec<TaskDto>> {
    let tasks = load_tasks(&state).await?;

    Ok(tasks
        .into_iter()
        .filter(|task| match filter {
            TaskFilter::All => true,
            TaskFilter::Overdue => task.status.as_deref() == Some("atrasada"),
            TaskFilter::Pending => task.status.as_deref() == Some("pendiente"),
            TaskFilter::Submitted => task.submitted,
            TaskFilter::Graded => task.grade.is_some(),
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
    load_schedule(&state).await
}

#[tauri::command]
pub async fn list_news(state: State<'_, AppState>) -> Cmd<Vec<NewsItemDto>> {
    let rows: Vec<NewsRow> = sqlx::query_as(
        "SELECT a.id, c.fullname AS course_title, a.title, a.body, a.published_at
         FROM announcements a
         LEFT JOIN courses c ON c.id = a.course_id
         ORDER BY COALESCE(a.published_at, 0) DESC
         LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    let mut items: Vec<NewsItemDto> = rows
        .into_iter()
        .map(|row| NewsItemDto {
            id: row.id,
            course: row.course_title,
            title: row.title,
            body: row.body,
            published_at: row.published_at,
            kind: "anuncio".into(),
        })
        .collect();

    for resource in load_resources(&state)
        .await?
        .into_iter()
        .filter(|r| r.is_new)
    {
        items.push(NewsItemDto {
            id: resource.id,
            course: resource.course_id,
            title: resource.title,
            body: String::new(),
            published_at: None,
            kind: "recurso".into(),
        });
    }

    Ok(items)
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

    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT kind, entity_id, title, body FROM search_index
         WHERE search_index MATCH ? LIMIT 50",
    )
    .bind(format!("\"{cleaned}\""))
    .fetch_all(&state.pool)
    .await
    .map_err(fail)?;

    let items: Vec<SearchHitDto> = rows
        .into_iter()
        .map(|(kind, id, title, body)| SearchHitDto {
            kind,
            id,
            title,
            body,
        })
        .collect();

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
            _ => {}
        }
    }

    if settings.downloads_dir.is_empty() {
        settings.downloads_dir = std::env::var("USERPROFILE")
            .map(|home| format!("{home}\\Downloads\\PanelRamos"))
            .unwrap_or_default();
    }

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: SettingsDto) -> Cmd<()> {
    let pairs = [
        (
            "sync_interval_minutes",
            settings.sync_interval_minutes.to_string(),
        ),
        ("downloads_dir", settings.downloads_dir),
        ("theme", settings.theme),
        ("notifications", settings.notifications.to_string()),
    ];

    for (key, value) in pairs {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&state.pool)
        .await
        .map_err(fail)?;
    }

    Ok(())
}
