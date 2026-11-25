use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use rust_app_lib::auth::Source;
use rust_app_lib::banner::models::{BannerGrade, BannerOption};
use rust_app_lib::db::models::{Course, CourseGroup};
use rust_app_lib::db::repo::Repo;
use rust_app_lib::db::{connect, migrate};
use rust_app_lib::ellucian::models::{EllucianClass, InstructionalEvent};
use rust_app_lib::error::AppError;
use rust_app_lib::http::rate_limit::RateLimiter;
use rust_app_lib::moodle::models::{
    MoodleActivity, MoodleAnnouncement, MoodleAssignment, MoodleCourse, MoodleEvent, MoodleFile,
    MoodleGradeTotal, MoodleSection,
};
use rust_app_lib::state::AppState;
use rust_app_lib::sync::downloads::safe_name;
use rust_app_lib::sync::sources::{EllucianSource, MoodleActivityPage, MoodleSource, SyncDeps};
use rust_app_lib::sync::{BannerSource, SyncOrchestrator, SyncStatusKind, SyncTrigger};

#[derive(Default)]
struct FakeMoodle {
    calls: AtomicUsize,
    fail_announcements: bool,
}

#[async_trait]
impl MoodleSource for FakeMoodle {
    async fn health(&self) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn courses(&self) -> Result<Vec<MoodleCourse>, AppError> {
        Ok(vec![MoodleCourse {
            id: 821_065,
            fullname: "Semestre Pregrado 2026 - 2 Minería de Datos (IGR-00167-00G)".into(),
            shortname: "30061.202620".into(),
            category: "PREGRADO".into(),
        }])
    }

    async fn calendar(&self) -> Result<Vec<MoodleEvent>, AppError> {
        Ok(Vec::new())
    }

    async fn sections(&self, _course_id: i64) -> Result<Vec<MoodleSection>, AppError> {
        let round = self.calls.fetch_add(1, Ordering::SeqCst);
        let mut activities = vec![MoodleActivity {
            cmid: 100,
            name: "Taller 1".into(),
            modtype: "assign".into(),
            url: "https://aulasvirtuales.santotomas.cl/mod/assign/view.php?id=100".into(),
        }];

        if round >= 1 {
            activities.push(MoodleActivity {
                cmid: 200,
                name: "Taller 2".into(),
                modtype: "assign".into(),
                url: "https://aulasvirtuales.santotomas.cl/mod/assign/view.php?id=200".into(),
            });
        }

        Ok(vec![MoodleSection {
            id: 3,
            name: "Unidad 2".into(),
            activities,
        }])
    }

    async fn activity(&self, _url: &str) -> Result<MoodleActivityPage, AppError> {
        Ok(MoodleActivityPage {
            assignment: Some(MoodleAssignment {
                submitted: false,
                graded: false,
                status_raw: "Todavía no se han realizado envíos".into(),
                due_at: Some(1_900_000_000),
                cutoff_at: None,
            }),
            description: "Enunciado del taller".into(),
            files: vec![MoodleFile {
                name: "enunciado.pdf".into(),
                url: "https://aulasvirtuales.santotomas.cl/pluginfile.php/1/mod_assign/introattachment/0/enunciado.pdf"
                    .into(),
                mimetype: Some("application/pdf".into()),
                size: Some(1024),
            }],
        })
    }

    async fn announcements(&self, _course_id: i64) -> Result<Vec<MoodleAnnouncement>, AppError> {
        if self.fail_announcements {
            return Err(AppError::Parse("foro sin respuesta".into()));
        }
        Ok(Vec::new())
    }

    async fn grade_total(&self, _course_id: i64) -> Result<Option<MoodleGradeTotal>, AppError> {
        Ok(Some(MoodleGradeTotal {
            total: 5.5,
            max_total: Some(7.0),
        }))
    }

    async fn download(&self, _url: &str, dest: &Path) -> Result<std::path::PathBuf, AppError> {
        Ok(dest.to_path_buf())
    }
}

struct FakeEllucian;

#[async_trait]
impl EllucianSource for FakeEllucian {
    async fn health(&self) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn classes(&self) -> Result<Vec<EllucianClass>, AppError> {
        Ok(vec![EllucianClass {
            section_id: "sec-1".into(),
            crn: "30061".into(),
            subject: "IGR".into(),
            number: "00167".into(),
            title: "Minería de Datos".into(),
            term: "202620".into(),
            term_title: "Semestre Pregrado 2026 - 2".into(),
            moodle_shortname: None,
        }])
    }

    async fn schedule(
        &self,
        _classes: &[EllucianClass],
    ) -> Result<Vec<InstructionalEvent>, AppError> {
        Ok(vec![InstructionalEvent {
            id: "ev-1".into(),
            section_id: Some("sec-1".into()),
            title: Some("Minería de Datos".into()),
            term: Some("202620".into()),
            day_of_week: 5,
            start_time: "19:21".into(),
            end_time: "20:00".into(),
            room: Some("Sala 401".into()),
            building: Some("Dieciocho N° 1191".into()),
            instructor: Some("PABLO LILLO GALLARDO".into()),
        }])
    }
}

struct FakeBanner;

#[async_trait]
impl BannerSource for FakeBanner {
    async fn health(&self) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn periods(&self) -> Result<Vec<BannerOption>, AppError> {
        Ok(vec![
            BannerOption {
                code: "-1".into(),
                description: "Todos los periodos".into(),
            },
            BannerOption {
                code: "202620".into(),
                description: "Semestre Pregrado 2026 - 2".into(),
            },
        ])
    }

    async fn levels(&self, _term: &str) -> Result<Vec<BannerOption>, AppError> {
        Ok(vec![BannerOption {
            code: "02".into(),
            description: "Profesional".into(),
        }])
    }

    async fn grades(&self, term: &str, _level: &str) -> Result<Vec<BannerGrade>, AppError> {
        Ok(vec![BannerGrade {
            subject: "IGR".into(),
            number: "00167".into(),
            crn: "30061".into(),
            section: "00G".into(),
            title: "Minería De Datos".into(),
            final_grade: None,
            midterm_grade: Some("6,2".into()),
            history_final_grade: None,
            calculated_final_grade: None,
            term: term.into(),
            term_title: "Semestre Pregrado 2026 - 2".into(),
            level: "02".into(),
        }])
    }
}

async fn test_state_with_moodle(moodle: Arc<dyn MoodleSource>) -> AppState {
    let pool = connect(Path::new(":memory:")).await.unwrap();
    migrate(&pool).await.unwrap();
    let repo = Repo::new(pool.clone());

    let deps = SyncDeps {
        moodle,
        ellucian: Arc::new(FakeEllucian),
        banner: Arc::new(FakeBanner),
        repo: repo.clone(),
        limiter: Arc::new(RateLimiter::new(6, 600)),
        emitter: None,
    };

    AppState::new(
        pool,
        repo,
        Arc::new(SyncOrchestrator::new(deps)),
        std::env::temp_dir(),
    )
}

async fn test_state() -> AppState {
    test_state_with_moodle(Arc::new(FakeMoodle::default())).await
}

#[test]
fn limiter_counts_run_requests() {
    let limiter = RateLimiter::new(6, 10);
    assert_eq!(limiter.count(), 0);
}

#[tokio::test]
async fn status_is_terminal_after_run() {
    let state = test_state().await;
    state.sync.run(SyncTrigger::Manual).await.unwrap();

    let status = state.sync.status();
    assert_eq!(status.state, SyncStatusKind::Ok);
    assert_eq!(status.progress, 100);
}

#[tokio::test]
async fn announcement_failure_does_not_abort_the_course() {
    let moodle = Arc::new(FakeMoodle {
        calls: AtomicUsize::new(0),
        fail_announcements: true,
    });
    let state = test_state_with_moodle(moodle).await;

    let run = state.sync.run(SyncTrigger::Manual).await.unwrap();

    assert_eq!(run.status, SyncStatusKind::Partial);
    assert_eq!(state.repo.course_count().await.unwrap(), 1);
}

#[tokio::test]
async fn start_returns_running_and_finishes() {
    let state = test_state().await;
    let status = state.sync.start(SyncTrigger::Manual);
    assert_eq!(status.state, SyncStatusKind::Running);

    for _ in 0..200 {
        if state.sync.status().state != SyncStatusKind::Running {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert_eq!(state.sync.status().state, SyncStatusKind::Ok);
}

#[tokio::test]
async fn full_run_saves_catalog() {
    let state = test_state().await;
    let run = state
        .sync
        .run(rust_app_lib::sync::SyncTrigger::Manual)
        .await
        .unwrap();

    assert_eq!(run.status, SyncStatusKind::Ok);
    assert_eq!(state.repo.course_count().await.unwrap(), 1);

    let groups = state.repo.list_course_groups().await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].code, "IGR-00167");
    assert_eq!(groups[0].name, "Minería de Datos");
    assert_eq!(groups[0].sections.len(), 1);

    let acknowledged: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM activities WHERE acknowledged_at IS NOT NULL AND archived_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap();
    assert_eq!(acknowledged, 1, "one activity, already seen");

    let files: i64 = sqlx::query_scalar("SELECT count(*) FROM files")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    assert_eq!(files, 1);

    let events: i64 = sqlx::query_scalar("SELECT count(*) FROM schedule_events")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    assert_eq!(events, 1, "the Ellucian schedule");

    let totals: i64 = sqlx::query_scalar("SELECT count(*) FROM grade_totals")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    assert_eq!(totals, 2, "Moodle and Banner totals kept apart");
}

#[tokio::test]
async fn expired_session_keeps_data_and_marks_partial() {
    let state = test_state().await;
    state
        .sync
        .run(rust_app_lib::sync::SyncTrigger::Manual)
        .await
        .unwrap();

    let run = state.sync.run_with_failure(Source::Moodle).await.unwrap();

    assert_eq!(run.status, SyncStatusKind::Partial);
    assert!(state.repo.course_count().await.unwrap() > 0);
    assert_eq!(
        state.repo.course_count().await.unwrap(),
        1,
        "the previous course is still there"
    );
}

#[tokio::test]
async fn second_run_marks_only_new_items() {
    let state = test_state().await;
    state
        .sync
        .run(rust_app_lib::sync::SyncTrigger::Manual)
        .await
        .unwrap();
    state
        .sync
        .run(rust_app_lib::sync::SyncTrigger::Manual)
        .await
        .unwrap();

    let old: Option<String> =
        sqlx::query_scalar("SELECT acknowledged_at FROM activities WHERE id = 'assign:100'")
            .fetch_one(&state.pool)
            .await
            .unwrap();
    let new: Option<String> =
        sqlx::query_scalar("SELECT acknowledged_at FROM activities WHERE id = 'assign:200'")
            .fetch_one(&state.pool)
            .await
            .unwrap();

    assert!(old.is_some(), "the known activity stays acknowledged");
    assert!(new.is_none(), "the new one stays new");

    let events: i64 = sqlx::query_scalar("SELECT count(*) FROM events WHERE kind = 'activity'")
        .fetch_one(&state.pool)
        .await
        .unwrap();
    assert_eq!(events, 1, "only the event of the new activity");
}

#[tokio::test]
async fn archives_disappeared_items_only_if_course_synced() {
    let state = test_state().await;
    let now = chrono::Utc::now();
    let stale = (now - chrono::Duration::days(8)).to_rfc3339();
    let fresh = now.to_rfc3339();

    let repo = &state.repo;
    repo.upsert_catalog(
        &[CourseGroup {
            id: "IGR-00167".into(),
            code: "IGR-00167".into(),
            name: "Minería de Datos".into(),
        }],
        &[
            Course {
                id: "moodle:1".into(),
                group_id: Some("IGR-00167".into()),
                source: "moodle".into(),
                moodle_id: Some(1),
                banner_crn: None,
                shortname: Some("30061.202620".into()),
                fullname: "Minería de Datos".into(),
                section: Some("00G".into()),
                term: Some("202620".into()),
                teacher: None,
                moodle_url: None,
                banner_url: None,
                last_synced_at: Some(fresh.clone()),
            },
            Course {
                id: "moodle:2".into(),
                group_id: None,
                source: "moodle".into(),
                moodle_id: Some(2),
                banner_crn: None,
                shortname: Some("30060.202620".into()),
                fullname: "Ramo Caído".into(),
                section: None,
                term: None,
                teacher: None,
                moodle_url: None,
                banner_url: None,
                last_synced_at: Some(stale.clone()),
            },
        ],
    )
    .await
    .unwrap();

    for (id, course) in [("assign:1", "moodle:1"), ("assign:2", "moodle:2")] {
        sqlx::query(
            "INSERT INTO activities (id, course_id, name, modtype, last_seen_at)
             VALUES (?, ?, 'Vieja', 'assign', ?)",
        )
        .bind(id)
        .bind(course)
        .bind(&stale)
        .execute(&state.pool)
        .await
        .unwrap();
    }

    let archived = rust_app_lib::sync::archive_stale(&state.pool, now)
        .await
        .unwrap();
    assert_eq!(archived, 1);

    let archived_first: Option<String> =
        sqlx::query_scalar("SELECT archived_at FROM activities WHERE id = 'assign:1'")
            .fetch_one(&state.pool)
            .await
            .unwrap();
    let archived_second: Option<String> =
        sqlx::query_scalar("SELECT archived_at FROM activities WHERE id = 'assign:2'")
            .fetch_one(&state.pool)
            .await
            .unwrap();

    assert!(archived_first.is_some(), "the course that synced archives");
    assert!(
        archived_second.is_none(),
        "the failed course archives nothing"
    );
}

#[test]
fn file_name_is_safe() {
    assert_eq!(safe_name("../../x.pdf"), "x.pdf");
    assert_eq!(safe_name("a\\b.pdf"), "b.pdf");
    assert_eq!(safe_name(".."), "archivo");

    let long = format!("{}.pdf", "a".repeat(300));
    let safe = safe_name(&long);
    assert_eq!(safe.chars().count(), 120);
    assert!(safe.ends_with(".pdf"));
}
