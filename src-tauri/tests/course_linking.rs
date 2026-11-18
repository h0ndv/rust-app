use rust_app_lib::banner::models::BannerGrade;
use rust_app_lib::db::linking::link_courses;
use rust_app_lib::moodle::models::MoodleCourse;
use rust_app_lib::moodle::parser::parse_courses_json;

fn moodle_course(shortname: &str, fullname: &str) -> MoodleCourse {
    let id = shortname
        .split('.')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    MoodleCourse {
        id,
        fullname: fullname.to_string(),
        shortname: shortname.to_string(),
        category: "Semestre Pregrado 2026 - 2: PREGRADO".to_string(),
    }
}

fn banner_grade(subject: &str, number: &str, section: &str, crn: &str) -> BannerGrade {
    BannerGrade {
        subject: subject.to_string(),
        number: number.to_string(),
        crn: crn.to_string(),
        section: section.to_string(),
        title: "Minería De Datos".to_string(),
        final_grade: None,
        midterm_grade: None,
        history_final_grade: None,
        calculated_final_grade: None,
        term: "202620".to_string(),
        term_title: "Semestre Pregrado 2026 - 2".to_string(),
        level: "02".to_string(),
    }
}

#[test]
fn une_por_codigo_y_seccion() {
    let moodle = vec![moodle_course(
        "30061.202620",
        "Minería de Datos (IGR-00167-00G)",
    )];
    let banner = vec![banner_grade("IGR", "00167", "00G", "30061")];

    let links = link_courses(&moodle, &banner);
    assert_eq!(links.len(), 1);
    assert!(links[0].banner_crn.is_some());
    assert_eq!(links[0].banner_crn.as_deref(), Some("30061"));
    assert_eq!(links[0].moodle_course_id.as_deref(), Some("30061"));
}

#[test]
fn un_curso_sin_pareja_sigue_apareciendo() {
    let moodle = vec![moodle_course(
        "99999.202620",
        "Ramo Sin Banner (XXX-00000-00G)",
    )];
    let links = link_courses(&moodle, &[]);

    assert_eq!(links.len(), 1);
    assert!(links[0].banner_crn.is_none());
    assert_eq!(links[0].moodle_course_id.as_deref(), Some("99999"));
}

#[test]
fn un_curso_solo_en_banner_tambien_aparece() {
    let banner = vec![banner_grade("XXX", "00000", "00G", "12345")];
    let links = link_courses(&[], &banner);

    assert_eq!(links.len(), 1);
    assert!(links[0].moodle_course_id.is_none());
    assert_eq!(links[0].banner_crn.as_deref(), Some("12345"));
}

#[test]
fn el_shortname_gana_sobre_el_codigo_del_nombre() {
    // The name keeps an old section, but the shortname and the CRN
    // match: the union must use the shortname.
    let moodle = vec![moodle_course(
        "30061.202620",
        "Minería de Datos (IGR-00167-XXY)",
    )];
    let banner = vec![banner_grade("IGR", "00167", "00G", "30061")];

    let links = link_courses(&moodle, &banner);
    assert_eq!(links[0].banner_crn.as_deref(), Some("30061"));
}

#[test]
fn un_curso_de_otro_semestre_no_se_une() {
    let moodle = vec![moodle_course(
        "30061.202520",
        "Minería de Datos (IGR-00167-00G)",
    )];
    let banner = vec![banner_grade("IGR", "00167", "00G", "30061")];

    let links = link_courses(&moodle, &banner);
    assert!(links[0].banner_crn.is_none());
}

#[test]
fn los_fixtures_reales_se_unan_completos() {
    let moodle = parse_courses_json(include_str!("fixtures/moodle/courses.json")).unwrap();
    let banner = rust_app_lib::banner::parser::parse_courses_response(include_str!(
        "fixtures/banner/courses.json"
    ))
    .unwrap();

    assert_eq!(moodle.len(), 8);
    assert_eq!(banner.len(), 8);

    let links = link_courses(&moodle, &banner);
    assert_eq!(links.len(), 8);
    assert!(
        links.iter().all(|link| link.banner_crn.is_some()),
        "los ocho ramos de Moodle tienen pareja en Banner"
    );
    assert!(
        links.iter().all(|link| link.moodle_course_id.is_some()),
        "y ninguno queda suelto"
    );
}

async fn catalog() -> rust_app_lib::db::repo::Repo {
    use rust_app_lib::db::linking::{course_groups, course_rows};
    use rust_app_lib::db::repo::Repo;
    use rust_app_lib::db::{connect, migrate};

    let pool = connect(std::path::Path::new(":memory:")).await.unwrap();
    migrate(&pool).await.unwrap();

    let moodle = parse_courses_json(include_str!("fixtures/moodle/courses.json")).unwrap();
    let banner = rust_app_lib::banner::parser::parse_courses_response(include_str!(
        "fixtures/banner/courses.json"
    ))
    .unwrap();

    let links = link_courses(&moodle, &banner);
    let courses = course_rows(&moodle, &banner, &links);
    let groups = course_groups(&courses);

    let repo = Repo::new(pool);
    repo.upsert_catalog(&groups, &courses).await.unwrap();
    repo
}

#[tokio::test]
async fn el_grupo_agrupa_las_dos_secciones_y_muestra_el_vencimiento() {
    use rust_app_lib::db::linking::course_groups;

    let repo = catalog().await;
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO activities (id, course_id, name, modtype, due_at)
         VALUES ('assign:1', 'moodle:821065', 'Taller', 'assign', ?)",
    )
    .bind(now + 86_400)
    .execute(repo.pool())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO activities (id, course_id, name, modtype, due_at)
         VALUES ('assign:2', 'moodle:821065', 'Vieja', 'assign', ?)",
    )
    .bind(now - 86_400)
    .execute(repo.pool())
    .await
    .unwrap();

    let groups = repo.list_course_groups().await.unwrap();
    assert_eq!(groups.len(), 6);

    let mining = groups
        .iter()
        .find(|group| group.code == "IGR-00167")
        .unwrap();
    assert_eq!(mining.name, "Minería de Datos");
    assert_eq!(mining.sections.len(), 2, "cátedra y laboratorio");

    let lab = mining
        .sections
        .iter()
        .find(|section| section.banner_crn.as_deref() == Some("30061"))
        .unwrap();
    assert_eq!(lab.next_due_at, Some(now + 86_400));
    assert_eq!(lab.pending_tasks, 2);

    assert_eq!(course_groups(&[]).len(), 0);
}

#[tokio::test]
async fn el_detalle_del_ramo_trae_tareas_horario_y_recursos() {
    let repo = catalog().await;
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO activities (id, course_id, name, modtype, due_at)
         VALUES ('assign:9', 'moodle:821065', 'Actividad 2.6', 'assign', ?)",
    )
    .bind(now + 3_600)
    .execute(repo.pool())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO files (id, course_id, name, url, kind)
         VALUES ('file:1', 'moodle:821065', 'enunciado.pdf', 'https://x/1', 'archivo')",
    )
    .execute(repo.pool())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO schedule_events (id, course_id, title, day_of_week, start_time, end_time, room)
         VALUES ('ev:1', 'moodle:821065', 'Minería de Datos', 5, '19:21', '20:00', 'Sala 401')",
    )
    .execute(repo.pool())
    .await
    .unwrap();

    let detail = repo.get_course_detail("IGR-00167").await.unwrap();
    assert_eq!(detail.group.code, "IGR-00167");
    assert_eq!(detail.tasks.len(), 1);
    assert_eq!(detail.resources.len(), 1);
    assert_eq!(detail.schedule.len(), 1);
    assert_eq!(detail.schedule[0].room.as_deref(), Some("Sala 401"));

    assert!(matches!(
        repo.get_course_detail("XXX-00000").await,
        Err(rust_app_lib::error::AppError::NotFound)
    ));
}
