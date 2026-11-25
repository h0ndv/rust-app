use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::auth::Source;
use crate::banner::models::BannerGrade;
use crate::db::linking::{course_groups, course_rows, link_courses, shortname_term};
use crate::db::models::{
    Activity, Announcement, FileItem, GradeItem, GradeTotal, ScheduleEvent, SourceStatus,
};
use crate::ellucian::models::{EllucianClass, InstructionalEvent};
use crate::error::AppError;
use crate::moodle::models::{MoodleActivity, MoodleCourse, MoodleEvent, MoodleFile};
use crate::sync::diff;
use crate::sync::sources::{MoodleActivityPage, SyncDeps};
use crate::sync::{SyncEvent, SyncRun, SyncStatus, SyncStatusKind, SyncTrigger};

const PROGRESS_STEPS: u8 = 9;

fn progress(step: u8) -> u8 {
    ((u32::from(step) * 100) / u32::from(PROGRESS_STEPS)).min(100) as u8
}

pub struct SyncOrchestrator {
    deps: SyncDeps,
    status: Mutex<SyncStatus>,
    cancel: Mutex<CancellationToken>,
    force_failure: Mutex<Option<Source>>,
    running: AtomicBool,
    last_run: Mutex<Option<std::time::Instant>>,
}

impl SyncOrchestrator {
    pub fn new(deps: SyncDeps) -> Self {
        Self {
            deps,
            status: Mutex::new(SyncStatus {
                state: SyncStatusKind::Ok,
                source: None,
                progress: 0,
                requests: 0,
                errors: Vec::new(),
            }),
            cancel: Mutex::new(CancellationToken::new()),
            force_failure: Mutex::new(None),
            running: AtomicBool::new(false),
            last_run: Mutex::new(Some(std::time::Instant::now())),
        }
    }

    pub fn status(&self) -> SyncStatus {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or(SyncStatus {
                state: SyncStatusKind::Ok,
                source: None,
                progress: 0,
                requests: 0,
                errors: Vec::new(),
            })
    }

    pub fn cancel(&self) {
        if let Ok(token) = self.cancel.lock() {
            token.cancel();
        }
    }

    fn try_begin(&self) -> bool {
        !self.running.swap(true, Ordering::SeqCst)
    }

    fn end(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Starts a run without waiting for it. The returned snapshot is the
    /// initial `running` state; the rest arrives through the events.
    pub fn start(self: &Arc<Self>, trigger: SyncTrigger) -> SyncStatus {
        if !self.try_begin() {
            return self.status();
        }

        self.publish(0, None, Vec::new());
        let status = self.status();
        let this = Arc::clone(self);

        let _task = tauri::async_runtime::spawn(async move {
            if let Err(error) = this.execute(trigger).await {
                tracing::warn!(%error, "manual sync failed");
            }
            this.end();
        });

        status
    }

    pub async fn run(&self, trigger: SyncTrigger) -> Result<SyncRun, AppError> {
        if !self.try_begin() {
            return Err(AppError::Internal("a sync is already running".into()));
        }
        let result = self.execute(trigger).await;
        self.end();
        result
    }

    async fn execute(&self, trigger: SyncTrigger) -> Result<SyncRun, AppError> {
        let result = self.run_inner(trigger).await;

        if let Ok(mut last) = self.last_run.lock() {
            *last = Some(std::time::Instant::now());
        }

        match result {
            Ok(run) => Ok(run),
            Err(error) => {
                let now = Utc::now().to_rfc3339();
                let run = SyncRun {
                    id: Uuid::new_v4().to_string(),
                    status: SyncStatusKind::Failed,
                    requests: self.deps.limiter.count(),
                    started_at: now.clone(),
                    finished_at: Some(now),
                };
                self.publish_final(
                    SyncStatusKind::Failed,
                    self.deps.limiter.count(),
                    vec![error.to_string()],
                );
                self.emit(SyncEvent::Finished(run.clone()));
                Err(error)
            }
        }
    }

    fn publish_final(&self, state: SyncStatusKind, requests: u32, errors: Vec<String>) {
        let status = SyncStatus {
            state,
            source: None,
            progress: 100,
            requests,
            errors,
        };
        if let Ok(mut guard) = self.status.lock() {
            *guard = status;
        }
    }

    /// Test seam: forces the failure of a source without touching the clients.
    pub async fn run_with_failure(&self, source: Source) -> Result<SyncRun, AppError> {
        if let Ok(mut forced) = self.force_failure.lock() {
            *forced = Some(source);
        }
        let result = self.run(SyncTrigger::Manual).await;
        if let Ok(mut forced) = self.force_failure.lock() {
            *forced = None;
        }
        result
    }

    /// One-minute heartbeat: reads the interval from the settings and runs when
    /// the elapsed time since the last run reaches it.
    pub fn spawn_scheduler(self: &Arc<Self>) {
        let this = Arc::clone(self);

        tauri::async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(60));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            ticker.tick().await;

            loop {
                ticker.tick().await;

                let minutes = this.deps.repo.sync_interval_minutes().await.unwrap_or(30);
                let interval = Duration::from_secs(minutes.saturating_mul(60));
                let due = this
                    .last_run
                    .lock()
                    .map(|last| {
                        last.as_ref()
                            .map(|value| value.elapsed() >= interval)
                            .unwrap_or(true)
                    })
                    .unwrap_or(true);

                if !due {
                    continue;
                }

                if let Err(error) = this.run(SyncTrigger::Scheduled).await {
                    tracing::warn!(%error, "scheduled sync failed");
                }
            }
        });
    }

    async fn run_inner(&self, _trigger: SyncTrigger) -> Result<SyncRun, AppError> {
        let started_at = Utc::now().to_rfc3339();
        let run_started = std::time::Instant::now();
        let run_id = Uuid::new_v4().to_string();
        let token = self.fresh_token();
        let repo = &self.deps.repo;

        self.deps.limiter.reset_run();
        self.publish(0, None, Vec::new());

        // An interrupted run leaves `running` forever: it is closed
        // as failed when the next one starts.
        let _ = repo.finish_stale_runs(&started_at).await;

        repo.insert_sync_run(&crate::db::models::SyncRun {
            id: run_id.clone(),
            started_at: started_at.clone(),
            finished_at: None,
            status: "running".into(),
            requests: 0,
            errors_json: None,
        })
        .await?;

        let mut errors: Vec<String> = Vec::new();
        let mut successes = 0usize;
        let mut step = 0u8;

        for source in Source::ALL {
            let healthy = self.check_health(source).await;
            if !healthy {
                errors.push(format!("{}: no session", source.as_str()));
            }
            repo.upsert_source_status(&SourceStatus {
                source: source.as_str().to_string(),
                state: if healthy { "ok" } else { "failed" }.to_string(),
                last_checked_at: Some(Utc::now().to_rfc3339()),
                last_error: (!healthy).then(|| "no session".to_string()),
            })
            .await?;
        }
        step += 1;
        self.publish(progress(step), None, errors.clone());

        let mut moodle_courses: Vec<MoodleCourse> = Vec::new();
        let mut calendar: HashMap<i64, i64> = HashMap::new();

        if let Err(error) = self.forced(Source::Moodle) {
            errors.push(format!("moodle: {error}"));
        } else {
            match self.deps.moodle.courses().await {
                Ok(courses) => {
                    moodle_courses = courses;
                    successes += 1;
                }
                Err(error) => errors.push(format!("moodle cursos: {error}")),
            }
            match self.deps.moodle.calendar().await {
                Ok(events) => {
                    calendar = due_dates(&events);
                }
                Err(error) => errors.push(format!("moodle calendario: {error}")),
            }
        }
        step += 1;
        self.publish(progress(step), Some(Source::Moodle), errors.clone());

        let mut synced_courses: Vec<String> = Vec::new();
        for course in &moodle_courses {
            if token.is_cancelled() {
                errors.push("cancelada".into());
                break;
            }
            if let Err(error) = self.sync_course(course, &calendar, &mut errors).await {
                errors.push(format!("moodle curso {}: {error}", course.id));
                continue;
            }
            synced_courses.push(format!("moodle:{}", course.id));
        }
        step += 1;
        self.publish(progress(step), Some(Source::Moodle), errors.clone());

        let mut classes: Vec<EllucianClass> = Vec::new();
        let mut schedule: Vec<InstructionalEvent> = Vec::new();
        if let Err(error) = self.forced(Source::Ellucian) {
            errors.push(format!("ellucian: {error}"));
        } else {
            match self.deps.ellucian.classes().await {
                Ok(fetched) => {
                    classes = fetched;
                    successes += 1;
                    match self.deps.ellucian.schedule(&classes).await {
                        Ok(events) => schedule = events,
                        Err(error) => errors.push(format!("ellucian horario: {error}")),
                    }
                }
                Err(error) => errors.push(format!("ellucian clases: {error}")),
            }
        }
        step += 1;
        self.publish(progress(step), Some(Source::Ellucian), errors.clone());

        let mut banner_grades: Vec<BannerGrade> = Vec::new();
        if let Err(error) = self.forced(Source::Banner) {
            errors.push(format!("banner: {error}"));
        } else {
            match self.fetch_banner_grades().await {
                Ok(grades) => {
                    banner_grades = grades;
                    successes += 1;
                }
                Err(error) => errors.push(format!("banner notas: {error}")),
            }
        }
        step += 1;
        self.publish(progress(step), Some(Source::Banner), errors.clone());

        // A subject that comes only from Banner and already exists in the database is not duplicated.
        let links = link_courses(&moodle_courses, &banner_grades);
        let rows = course_rows(&moodle_courses, &banner_grades, &links);

        let mut by_crn: HashMap<String, String> = HashMap::new();
        let mut to_upsert = Vec::with_capacity(rows.len());

        for row in rows {
            if let Some(crn) = row.banner_crn.clone() {
                if row.moodle_id.is_none() {
                    match repo.course_id_by_crn(&crn).await {
                        Ok(Some(existing)) => {
                            by_crn.insert(crn, existing);
                            continue;
                        }
                        Ok(None) => {}
                        Err(error) => errors.push(format!("curso {crn}: {error}")),
                    }
                }
                by_crn.insert(crn, row.id.clone());
            }
            to_upsert.push(row);
        }

        let groups = course_groups(&to_upsert);

        if let Err(error) = repo.upsert_catalog(&groups, &to_upsert).await {
            errors.push(format!("catalog: {error}"));
        }
        step += 1;
        self.publish(progress(step), None, errors.clone());

        if let Err(error) = self
            .persist_grades(&banner_grades, &by_crn, &mut errors)
            .await
        {
            errors.push(format!("notas: {error}"));
        }
        step += 1;
        self.publish(progress(step), None, errors.clone());

        if let Err(error) = self
            .persist_schedule(&classes, &schedule, &by_crn, &mut errors)
            .await
        {
            errors.push(format!("horario: {error}"));
        }
        step += 1;
        self.publish(progress(step), None, errors.clone());

        let stamp = Utc::now().to_rfc3339();
        for course_id in &synced_courses {
            if let Err(error) = repo.set_course_synced(course_id, &stamp).await {
                errors.push(format!("curso {course_id}: {error}"));
            }
        }

        match diff::archive_stale(repo.pool(), Utc::now()).await {
            Ok(0) => {}
            Ok(archived) => tracing::debug!(archived, "archived stale items"),
            Err(error) => errors.push(format!("archivado: {error}")),
        }

        let state = if errors.is_empty() {
            SyncStatusKind::Ok
        } else if successes == 0 {
            SyncStatusKind::Failed
        } else {
            SyncStatusKind::Partial
        };

        let requests = self.deps.limiter.count();
        let finished_at = Utc::now().to_rfc3339();
        let errors_json =
            (!errors.is_empty()).then(|| serde_json::to_string(&errors).unwrap_or_default());

        repo.finish_sync_run(
            &run_id,
            status_str(state),
            i64::from(requests),
            errors_json.as_deref(),
            &finished_at,
        )
        .await?;

        let run = SyncRun {
            id: run_id,
            status: state,
            requests,
            started_at,
            finished_at: Some(finished_at),
        };

        if let Err(error) = repo.optimize().await {
            tracing::warn!(%error, "could not optimize the database after the sync");
        }

        tracing::info!(
            requests,
            elapsed_ms = run_started.elapsed().as_millis() as u64,
            "sync finished"
        );

        self.publish_final(state, requests, errors);
        self.emit(SyncEvent::Finished(run.clone()));

        Ok(run)
    }

    /// Sections, activities, announcements and total of a course. A failure here
    /// does not take down the run: the course is not marked as synced.
    async fn sync_course(
        &self,
        course: &MoodleCourse,
        calendar: &HashMap<i64, i64>,
        errors: &mut Vec<String>,
    ) -> Result<(), AppError> {
        let repo = &self.deps.repo;
        let course_row_id = format!("moodle:{}", course.id);
        let baseline = diff::is_baseline(repo.pool(), &course_row_id).await?;
        let now = Utc::now().to_rfc3339();
        let started = std::time::Instant::now();

        let content = self.deps.moodle.course_content(course.id).await?;

        for (index, section) in content.sections.iter().enumerate() {
            let section_row_id = format!("{course_row_id}:{}", section.id);
            repo.upsert_section(&crate::db::models::Section {
                id: section_row_id.clone(),
                course_id: course_row_id.clone(),
                name: section.name.clone(),
                position: index as i64,
            })
            .await?;

            for activity in &section.activities {
                if let Err(error) = self
                    .sync_activity(
                        &course_row_id,
                        &section_row_id,
                        activity,
                        calendar,
                        baseline,
                        &now,
                        errors,
                    )
                    .await
                {
                    errors.push(format!("moodle actividad {}: {error}", activity.cmid));
                }
            }
        }

        match content.announcements {
            Ok(items) => {
                for item in items {
                    let id = match item.id {
                        Some(discussion) => format!("moodle:{}:{discussion}", course.id),
                        None => format!("moodle:{}:{:x}", course.id, hash_text(&item.title)),
                    };
                    let is_new = !repo.announcement_exists(&id).await?;
                    repo.upsert_announcement(&Announcement {
                        id: id.clone(),
                        course_id: course_row_id.clone(),
                        title: item.title.clone(),
                        body: item.body.clone(),
                        url: item.url,
                        published_at: item.published_at,
                        first_seen_at: is_new.then(|| now.clone()),
                        acknowledged_at: (is_new && baseline).then(|| now.clone()),
                    })
                    .await?;

                    repo.index_entity("announcement", &id, &item.title, &item.body)
                        .await?;

                    if is_new && !baseline {
                        diff::mark_new(repo.pool(), "announcement", &id).await?;
                    }
                }
            }
            Err(error) => errors.push(format!("moodle anuncios {}: {error}", course.id)),
        }

        if let Some(term) = shortname_term(&course.shortname) {
            match self.deps.moodle.grade_total(course.id).await {
                Ok(Some(total)) => {
                    repo.upsert_grade_total(&GradeTotal {
                        course_id: course_row_id,
                        term,
                        total: Some(total.total),
                        max_total: total.max_total,
                        source: "moodle".into(),
                        updated_at: Some(now),
                    })
                    .await?;
                }
                Ok(None) => {}
                Err(error) => errors.push(format!("moodle total {}: {error}", course.id)),
            }
        }

        tracing::debug!(
            course_id = course.id,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "course synced"
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn sync_activity(
        &self,
        course_row_id: &str,
        section_row_id: &str,
        activity: &MoodleActivity,
        calendar: &HashMap<i64, i64>,
        baseline: bool,
        now: &str,
        errors: &mut Vec<String>,
    ) -> Result<(), AppError> {
        let repo = &self.deps.repo;
        if activity.modtype == "label" || activity.modtype.is_empty() {
            return Ok(());
        }

        let id = format!("{}:{}", activity.modtype, activity.cmid);
        let hash = content_hash(activity);
        let existing = repo.activity_state(&id).await?;
        let submitted_before = existing.as_ref().and_then(|(_, submitted)| *submitted);
        let needs_page = existing
            .as_ref()
            .map(|(saved, submitted)| {
                saved.as_deref() != Some(hash.as_str()) || submitted.is_none()
            })
            .unwrap_or(true);

        // A `resource` is the file itself: requesting its page downloads the whole
        // PDF and adds nothing.
        let is_resource = activity.modtype == "resource";

        let mut page = MoodleActivityPage::default();
        if needs_page && !is_resource && !activity.url.is_empty() {
            match self.deps.moodle.activity(&activity.url).await {
                Ok(fetched) => page = fetched,
                Err(error) => errors.push(format!("moodle pagina {}: {error}", activity.cmid)),
            }
        }

        let submitted = page
            .assignment
            .as_ref()
            .map(|assignment| assignment.submitted)
            .or(submitted_before);
        let graded = page.assignment.as_ref().map(|assignment| assignment.graded);
        let due_at = page
            .assignment
            .as_ref()
            .and_then(|assignment| assignment.due_at)
            .or_else(|| calendar.get(&activity.cmid).copied());
        let open_at = None;

        let is_new = existing.is_none();
        let status = activity_status(activity.modtype.as_str(), submitted, graded, due_at);

        repo.upsert_activity(&Activity {
            id: id.clone(),
            course_id: course_row_id.to_string(),
            section_id: Some(section_row_id.to_string()),
            cmid: Some(activity.cmid),
            name: activity.name.clone(),
            modtype: activity.modtype.clone(),
            url: (!activity.url.is_empty()).then(|| activity.url.clone()),
            due_at,
            open_at,
            cutoff_at: page.assignment.as_ref().and_then(|value| value.cutoff_at),
            status: Some(status.to_string()),
            submitted,
            graded,
            grade: None,
            max_grade: None,
            description: page.description.clone(),
            content_hash: Some(hash),
            first_seen_at: is_new.then(|| now.to_string()),
            acknowledged_at: (is_new && baseline).then(|| now.to_string()),
            archived_at: None,
            last_seen_at: Some(now.to_string()),
        })
        .await?;

        repo.index_entity("activity", &id, &activity.name, &page.description)
            .await?;

        if is_new && !baseline {
            diff::mark_new(repo.pool(), "activity", &id).await?;
        }

        let files: Vec<MoodleFile> = if is_resource && !activity.url.is_empty() {
            vec![MoodleFile {
                name: activity.name.clone(),
                url: activity.url.clone(),
                mimetype: None,
                size: None,
            }]
        } else {
            page.files.clone()
        };

        for file in &files {
            let file_id = format!("file:{:x}", hash_text(&file.url));
            let is_new = !repo.file_exists(&file_id).await?;
            repo.upsert_file(&FileItem {
                id: file_id.clone(),
                activity_id: Some(id.clone()),
                course_id: Some(course_row_id.to_string()),
                name: file.name.clone(),
                url: file.url.clone(),
                mimetype: file.mimetype.clone(),
                size: file.size,
                kind: "file".into(),
                sha256: None,
                local_path: None,
                first_seen_at: is_new.then(|| now.to_string()),
                acknowledged_at: (is_new && baseline).then(|| now.to_string()),
                archived_at: None,
                last_seen_at: Some(now.to_string()),
            })
            .await?;

            repo.index_entity("file", &file_id, &file.name, "").await?;

            if is_new && !baseline {
                diff::mark_new(repo.pool(), "file", &file_id).await?;
            }
        }

        Ok(())
    }

    async fn fetch_banner_grades(&self) -> Result<Vec<BannerGrade>, AppError> {
        let periods = self.deps.banner.periods().await?;
        let term = periods
            .iter()
            .find(|period| period.code != "-1")
            .map(|period| period.code.clone())
            .ok_or(AppError::NotFound)?;

        let levels = self.deps.banner.levels(&term).await?;
        let level = levels
            .first()
            .map(|level| level.code.clone())
            .unwrap_or_else(|| "02".to_string());

        self.deps.banner.grades(&term, &level).await
    }

    async fn persist_grades(
        &self,
        grades: &[BannerGrade],
        by_crn: &HashMap<String, String>,
        errors: &mut Vec<String>,
    ) -> Result<(), AppError> {
        let repo = &self.deps.repo;
        let now = Utc::now().to_rfc3339();

        for grade in grades {
            let Some(course_id) = by_crn.get(&grade.crn) else {
                continue;
            };
            let value = grade_number(grade.final_grade.as_deref());

            if let Err(error) = repo
                .upsert_grade_item(&GradeItem {
                    id: format!("banner:{}:{}", grade.term, grade.crn),
                    course_id: course_id.clone(),
                    term: grade.term.clone(),
                    name: grade.title.clone(),
                    category: None,
                    grade: value,
                    max_grade: Some(7.0),
                    weight: None,
                    contribution: None,
                    source: "banner".into(),
                    raw_json: serde_json::to_string(grade).ok(),
                })
                .await
            {
                errors.push(format!("nota {}: {error}", grade.crn));
            }

            if let Err(error) = repo
                .upsert_grade_total(&GradeTotal {
                    course_id: course_id.clone(),
                    term: grade.term.clone(),
                    total: value,
                    max_total: Some(7.0),
                    source: "banner".into(),
                    updated_at: Some(now.clone()),
                })
                .await
            {
                errors.push(format!("total {}: {error}", grade.crn));
            }
        }

        Ok(())
    }

    async fn persist_schedule(
        &self,
        classes: &[EllucianClass],
        events: &[InstructionalEvent],
        by_crn: &HashMap<String, String>,
        errors: &mut Vec<String>,
    ) -> Result<(), AppError> {
        let by_section: HashMap<&str, &str> = classes
            .iter()
            .filter_map(|class| {
                by_crn
                    .get(&class.crn)
                    .map(|course_id| (class.section_id.as_str(), course_id.as_str()))
            })
            .collect();

        for event in events {
            let course_id = event
                .section_id
                .as_deref()
                .and_then(|section| by_section.get(section))
                .map(|value| value.to_string());

            if let Err(error) = self
                .deps
                .repo
                .upsert_schedule_event(&ScheduleEvent {
                    id: event.id.clone(),
                    course_id,
                    title: event.title.clone().unwrap_or_default(),
                    day_of_week: Some(event.day_of_week),
                    start_time: Some(event.start_time.clone()),
                    end_time: Some(event.end_time.clone()),
                    room: event.room.clone(),
                    building: event.building.clone(),
                    instructor: event.instructor.clone(),
                    term: event.term.clone(),
                    raw_json: None,
                })
                .await
            {
                errors.push(format!("bloque {}: {error}", event.id));
            }
        }

        Ok(())
    }

    fn fresh_token(&self) -> CancellationToken {
        let mut guard = self
            .cancel
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.is_cancelled() {
            *guard = CancellationToken::new();
        }
        guard.clone()
    }

    fn is_forced(&self, source: Source) -> bool {
        self.force_failure
            .lock()
            .map(|forced| *forced == Some(source))
            .unwrap_or(false)
    }

    fn forced(&self, source: Source) -> Result<(), AppError> {
        if self.is_forced(source) {
            Err(AppError::Unauthorized)
        } else {
            Ok(())
        }
    }

    async fn check_health(&self, source: Source) -> bool {
        if self.is_forced(source) {
            return false;
        }
        let result = match source {
            Source::Moodle => self.deps.moodle.health().await,
            Source::Ellucian => self.deps.ellucian.health().await,
            Source::Banner => self.deps.banner.health().await,
        };
        matches!(result, Ok(true))
    }

    fn publish(&self, progress: u8, source: Option<Source>, errors: Vec<String>) {
        let status = SyncStatus {
            state: SyncStatusKind::Running,
            source,
            progress: progress.min(100),
            requests: self.deps.limiter.count(),
            errors,
        };

        if let Ok(mut guard) = self.status.lock() {
            *guard = status.clone();
        }
        self.emit(SyncEvent::Progress(status));
    }

    fn emit(&self, event: SyncEvent) {
        let Some(emitter) = &self.deps.emitter else {
            return;
        };

        let (name, payload) = match event {
            SyncEvent::Progress(status) => ("sync://progress", serde_json::to_value(status)),
            SyncEvent::Finished(run) => ("sync://finished", serde_json::to_value(run)),
        };

        if let Ok(payload) = payload {
            emitter(name, payload);
        }
    }
}

fn status_str(kind: SyncStatusKind) -> &'static str {
    match kind {
        SyncStatusKind::Running => "running",
        SyncStatusKind::Ok => "ok",
        SyncStatusKind::Partial => "partial",
        SyncStatusKind::Failed => "failed",
    }
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn content_hash(activity: &MoodleActivity) -> String {
    format!(
        "{:x}",
        hash_text(&format!(
            "{}|{}|{}",
            activity.name, activity.modtype, activity.url
        ))
    )
}

fn due_dates(events: &[MoodleEvent]) -> HashMap<i64, i64> {
    events
        .iter()
        .filter(|event| event.instance > 0 && event.timesort > 0)
        .map(|event| (event.instance, event.timesort))
        .collect()
}

fn grade_number(value: Option<&str>) -> Option<f64> {
    value
        .map(|text| text.trim().replace(',', "."))
        .and_then(|text| text.parse().ok())
}

fn activity_status(
    modtype: &str,
    submitted: Option<bool>,
    graded: Option<bool>,
    due_at: Option<i64>,
) -> &'static str {
    let now = Utc::now().timestamp();

    if graded == Some(true) {
        return "graded";
    }
    if submitted == Some(true) {
        return "submitted";
    }
    if matches!(modtype, "assign" | "quiz") {
        return match due_at {
            Some(due) if due < now => "overdue",
            Some(_) => "pending",
            None => "no_data",
        };
    }
    if due_at.is_some() {
        "unconfirmed"
    } else {
        "informational"
    }
}

#[cfg(test)]
mod tests {
    use super::activity_status;

    #[test]
    fn activity_status_uses_canonical_codes() {
        let past = Some(1);
        let future = Some(4_000_000_000);

        assert_eq!(
            activity_status("assign", Some(true), None, past),
            "submitted"
        );
        assert_eq!(activity_status("assign", None, Some(true), past), "graded");
        assert_eq!(activity_status("assign", None, None, past), "overdue");
        assert_eq!(activity_status("assign", None, None, future), "pending");
        assert_eq!(activity_status("assign", None, None, None), "no_data");
        assert_eq!(
            activity_status("resource", None, None, future),
            "unconfirmed"
        );
        assert_eq!(
            activity_status("resource", None, None, None),
            "informational"
        );
    }
}
