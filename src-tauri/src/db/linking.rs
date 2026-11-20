use regex::Regex;

use crate::banner::models::BannerGrade;
use crate::db::models::{Course, CourseGroup, CourseLink};
use crate::moodle::models::MoodleCourse;

pub const MOODLE_BASE: &str = "https://aulasvirtuales.santotomas.cl";
pub const BANNER_BASE: &str =
    "https://selferviceprd.santotomas.cl/StudentSelfService/studentGrades";

pub fn course_code(fullname: &str) -> Option<(String, String, String)> {
    let regex = Regex::new(r"([A-Z]{3})-(\d{5})-([A-Z0-9]{3})").expect("regex válida");
    let caps = regex.captures(fullname)?;
    Some((
        caps.get(1)?.as_str().to_string(),
        caps.get(2)?.as_str().to_string(),
        caps.get(3)?.as_str().to_string(),
    ))
}

fn shortname_parts(shortname: &str) -> (Option<String>, Option<String>) {
    match shortname.split_once('.') {
        Some((crn, term)) => {
            let crn = (!crn.is_empty() && crn.chars().all(|c| c.is_ascii_digit()))
                .then(|| crn.to_string());
            let term = (!term.is_empty()).then(|| term.to_string());
            (crn, term)
        }
        None => (None, None),
    }
}

fn term_matches(moodle: Option<&str>, banner: &str) -> bool {
    match moodle {
        Some(term) => banner.is_empty() || term == banner,
        None => true,
    }
}

/// A subject may exist only in Moodle, only in Banner or in both. None is
/// discarded: the Ramos list is built from the full union.
pub fn link_courses(moodle: &[MoodleCourse], banner: &[BannerGrade]) -> Vec<CourseLink> {
    let mut used = vec![false; banner.len()];
    let mut links = Vec::with_capacity(moodle.len() + banner.len());

    for course in moodle {
        let (crn, term) = shortname_parts(&course.shortname);
        let code = course_code(&course.fullname);

        let mut found = None;

        // First the CRN from the shortname: it survives a name change.
        if let Some(crn) = &crn {
            found = banner
                .iter()
                .position(|grade| &grade.crn == crn && term_matches(term.as_deref(), &grade.term));
        }

        if found.is_none()
            && let Some((subject, number, section)) = &code
        {
            found = banner.iter().position(|grade| {
                &grade.subject == subject
                    && &grade.number == number
                    && &grade.section == section
                    && term_matches(term.as_deref(), &grade.term)
            });
        }

        match found {
            Some(index) => {
                used[index] = true;
                let grade = &banner[index];
                links.push(CourseLink {
                    moodle_course_id: Some(course.id.to_string()),
                    banner_course_id: Some(format!("banner:{}", grade.crn)),
                    banner_crn: Some(grade.crn.clone()),
                });
            }
            None => links.push(CourseLink {
                moodle_course_id: Some(course.id.to_string()),
                banner_course_id: None,
                banner_crn: None,
            }),
        }
    }

    for (index, grade) in banner.iter().enumerate() {
        if used[index] {
            continue;
        }
        links.push(CourseLink {
            moodle_course_id: None,
            banner_course_id: Some(format!("banner:{}", grade.crn)),
            banner_crn: Some(grade.crn.clone()),
        });
    }

    links
}

pub fn course_rows(
    moodle: &[MoodleCourse],
    banner: &[BannerGrade],
    links: &[CourseLink],
) -> Vec<Course> {
    let mut rows = Vec::with_capacity(links.len());

    for link in links {
        let moodle_course = link
            .moodle_course_id
            .as_deref()
            .and_then(|id| moodle.iter().find(|course| course.id.to_string() == id));
        let grade = link
            .banner_crn
            .as_deref()
            .and_then(|crn| banner.iter().find(|grade| grade.crn == crn));

        let code = moodle_course
            .and_then(|course| course_code(&course.fullname))
            .map(|(subject, number, _)| format!("{subject}-{number}"))
            .or_else(|| grade.map(|grade| format!("{}-{}", grade.subject, grade.number)));

        let section = grade.map(|grade| grade.section.clone()).or_else(|| {
            moodle_course
                .and_then(|course| course_code(&course.fullname))
                .map(|(_, _, section)| section)
        });

        let term = moodle_course
            .and_then(|course| shortname_parts(&course.shortname).1)
            .or_else(|| grade.map(|grade| grade.term.clone()));

        let id = match (moodle_course, grade) {
            (Some(course), _) => format!("moodle:{}", course.id),
            (None, Some(grade)) => format!("banner:{}", grade.crn),
            (None, None) => continue,
        };

        rows.push(Course {
            id,
            group_id: code,
            source: if moodle_course.is_some() {
                "moodle".to_string()
            } else {
                "banner".to_string()
            },
            moodle_id: moodle_course.map(|course| course.id),
            banner_crn: grade.map(|grade| grade.crn.clone()),
            shortname: moodle_course.map(|course| course.shortname.clone()),
            fullname: moodle_course
                .map(|course| course.fullname.clone())
                .or_else(|| grade.map(|grade| grade.title.clone()))
                .unwrap_or_default(),
            section,
            term,
            teacher: None,
            moodle_url: moodle_course
                .map(|course| format!("{MOODLE_BASE}/course/view.php?id={}", course.id)),
            banner_url: grade.map(|_| BANNER_BASE.to_string()),
            last_synced_at: None,
        });
    }

    rows
}

fn clean_name(fullname: &str, code: &str) -> String {
    // The name carries the full code in parentheses: "(IGR-00167-00G)".
    let trailing = Regex::new(r"\s*\([A-Z]{3}-\d{5}-[A-Z0-9]{3}\)\s*$").expect("regex válida");
    let without_code = trailing.replace(fullname, "").to_string();
    let without_code = without_code.replace(&format!("({code})"), "");
    let prefix =
        Regex::new(r"(?i)^\s*semestre\s+pregrado\s+\d{4}\s*-\s*\d\s+").expect("regex válida");
    let name = prefix.replace(&without_code, "");
    name.trim().trim_end_matches('-').trim().to_string()
}

pub fn course_groups(courses: &[Course]) -> Vec<CourseGroup> {
    let mut groups: Vec<CourseGroup> = Vec::new();

    for course in courses {
        let Some(code) = &course.group_id else {
            continue;
        };
        if groups.iter().any(|group| &group.code == code) {
            continue;
        }
        groups.push(CourseGroup {
            id: code.clone(),
            code: code.clone(),
            name: clean_name(&course.fullname, code),
        });
    }

    groups
}

pub fn shortname_term(shortname: &str) -> Option<String> {
    shortname
        .split_once('.')
        .map(|(_, term)| term.to_string())
        .filter(|term| !term.is_empty())
}
