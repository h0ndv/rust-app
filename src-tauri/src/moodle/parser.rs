use chrono::{Local, NaiveDate, TimeZone};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

use crate::error::AppError;
use crate::moodle::models::{
    MoodleActivity, MoodleAnnouncement, MoodleAssignment, MoodleCourse, MoodleEvent, MoodleFile,
    MoodleGradeTotal, MoodleSection,
};

const LOGIN_ERRORS: [&str; 5] = [
    "invalidsesskey",
    "sessionexpired",
    "invalidlogin",
    "requirelogin",
    "sesskey",
];

const MONTHS: [(&str, u32); 12] = [
    ("enero", 1),
    ("febrero", 2),
    ("marzo", 3),
    ("abril", 4),
    ("mayo", 5),
    ("junio", 6),
    ("julio", 7),
    ("agosto", 8),
    ("septiembre", 9),
    ("octubre", 10),
    ("noviembre", 11),
    ("diciembre", 12),
];

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("selector CSS válido")
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn text_of(element: ElementRef<'_>) -> String {
    collapse(&element.text().collect::<String>())
}

/// Moodle wraps the AJAX responses in an array with `error` and `data`.
fn ajax_payload(raw: &str) -> Result<Value, AppError> {
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|e| AppError::Parse(format!("Moodle no devolvió JSON: {e}")))?;

    let first = parsed
        .get(0)
        .ok_or_else(|| AppError::Parse("respuesta AJAX vacía".into()))?;

    if first.get("error").and_then(Value::as_bool).unwrap_or(false) {
        let code = first
            .pointer("/exception/errorcode")
            .and_then(Value::as_str)
            .unwrap_or("error");
        return Err(if LOGIN_ERRORS.contains(&code) {
            AppError::Unauthorized
        } else {
            AppError::Parse(format!("Moodle devolvió error: {code}"))
        });
    }

    Ok(parsed)
}

fn as_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(|v| match v {
        Value::Number(number) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|f| f.fract() == 0.0)
                .map(|f| f as i64)
        }),
        Value::String(text) => text.parse().ok(),
        _ => None,
    })
}

fn as_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub fn parse_courses_json(raw: &str) -> Result<Vec<MoodleCourse>, AppError> {
    let payload = ajax_payload(raw)?;
    let courses = payload
        .pointer("/0/data/courses")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Parse("Moodle no devolvió cursos".into()))?;

    let mut out = Vec::with_capacity(courses.len());
    for course in courses {
        let Some(id) = as_i64(course.get("id")) else {
            continue;
        };
        out.push(MoodleCourse {
            id,
            fullname: as_string(course.get("fullname")),
            shortname: as_string(course.get("shortname")),
            category: as_string(course.get("coursecategory")),
        });
    }
    Ok(out)
}

pub fn parse_calendar_json(raw: &str) -> Result<Vec<MoodleEvent>, AppError> {
    let payload = ajax_payload(raw)?;
    let events = payload
        .pointer("/0/data/events")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Parse("Moodle no devolvió eventos".into()))?;

    let mut out = Vec::with_capacity(events.len());
    for event in events {
        let url = event.get("url").and_then(Value::as_str).map(str::to_string);
        out.push(MoodleEvent {
            name: as_string(event.get("name")),
            modulename: as_string(event.get("modulename")),
            instance: as_i64(event.get("instance")).unwrap_or(0),
            timesort: as_i64(event.get("timesort")).unwrap_or(0),
            overdue: event
                .get("overdue")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            url,
            course_id: as_i64(event.pointer("/course/id")).unwrap_or(0),
        });
    }
    Ok(out)
}

pub fn parse_section_numbers(html: &str) -> Vec<i64> {
    let doc = Html::parse_document(html);
    let mut numbers = std::collections::BTreeSet::from([0_i64]);

    let link = selector("a[href]");
    for anchor in doc.select(&link) {
        if let Some(number) = anchor
            .value()
            .attr("href")
            .and_then(section_of_href)
            .filter(|n| *n > 0)
        {
            numbers.insert(number);
        }
    }
    numbers.into_iter().collect()
}

fn section_of_href(href: &str) -> Option<i64> {
    let regex = Regex::new(r"section=(\d+)").expect("regex válida");
    regex
        .captures(href)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn section_name(doc: &Html, section: i64) -> String {
    let link = selector("a[href*=\"#tabs-tree-start\"]");
    for anchor in doc.select(&link) {
        let href = anchor.value().attr("href").unwrap_or("");
        if section_of_href(href) != Some(section) {
            continue;
        }
        let title = anchor.value().attr("title").unwrap_or("").trim();
        if !title.is_empty() {
            return title.to_string();
        }
        let text = text_of(anchor);
        if !text.is_empty() {
            return text;
        }
    }
    if section == 0 {
        "General".to_string()
    } else {
        format!("Sección {section}")
    }
}

/// A course page renders a single section, but the tabs
/// carry the name of all of them.
pub fn parse_course_html(html: &str, section: i64) -> Result<MoodleSection, AppError> {
    if html.trim().is_empty() {
        return Err(AppError::Parse("HTML vacío".into()));
    }

    let doc = Html::parse_document(html);
    let item_selector = selector("li.activity");
    let name_selector = selector("[data-activityname]");
    let anchor_selector = selector("a[href]");
    let module_regex = Regex::new(r"^module-(\d+)$").expect("regex válida");

    let mut activities = Vec::new();
    for item in doc.select(&item_selector) {
        let Some(cmid) = item
            .value()
            .attr("id")
            .and_then(|id| module_regex.captures(id))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<i64>().ok())
        else {
            continue;
        };

        let class = item.value().attr("class").unwrap_or("");
        let modtype = class
            .split_whitespace()
            .find_map(|token| token.strip_prefix("modtype_"))
            .unwrap_or_default()
            .to_string();

        let url = item
            .select(&anchor_selector)
            .filter_map(|anchor| anchor.value().attr("href"))
            .find(|href| href.contains("/mod/") || href.contains("/course/mod.php"))
            .unwrap_or_default()
            .to_string();

        let name = item
            .select(&name_selector)
            .next()
            .and_then(|element| element.value().attr("data-activityname"))
            .map(collapse)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                item.select(&anchor_selector)
                    .find(|anchor| {
                        anchor
                            .value()
                            .attr("href")
                            .is_some_and(|h| h.contains("/mod/"))
                    })
                    .map(text_of)
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_default();

        activities.push(MoodleActivity {
            cmid,
            name,
            modtype,
            url,
        });
    }

    Ok(MoodleSection {
        id: section,
        name: section_name(&doc, section),
        activities,
    })
}

pub fn parse_news_forum_url(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let forum = selector("li.activity.modtype_forum");
    let name_selector = selector("[data-activityname]");
    let preferred = selector("a.aalink");
    let fallback = selector("a[href*=\"mod/forum/view.php\"]");

    for item in doc.select(&forum) {
        let name = item
            .select(&name_selector)
            .next()
            .and_then(|element| element.value().attr("data-activityname"))
            .unwrap_or_default()
            .to_lowercase();

        let key = ["novedad", "anuncio", "aviso", "announcement"];
        if !key.iter().any(|key| name.contains(key)) {
            continue;
        }

        let anchor = item
            .select(&preferred)
            .next()
            .or_else(|| item.select(&fallback).next());
        if let Some(href) = anchor.and_then(|a| a.value().attr("href")) {
            return Some(href.to_string());
        }
    }
    None
}

fn discussion_id(href: &str) -> Option<i64> {
    let regex = Regex::new(r"d=(\d+)").expect("regex válida");
    regex
        .captures(href)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn timestamp(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|date| date.timestamp())
}

pub fn parse_announcements_html(html: &str) -> Result<Vec<MoodleAnnouncement>, AppError> {
    let doc = Html::parse_document(html);
    let mut out = Vec::new();

    let post_selector = selector(".forumpost");
    let title_selector = selector("h3, h4, .subject");
    let body_selector = selector(".posting, .post-content, .postmessage");
    let link_selector = selector("a[href*=\"discuss.php\"]");
    let time_selector = selector("time[datetime]");

    for post in doc.select(&post_selector) {
        let title = post
            .select(&title_selector)
            .next()
            .map(text_of)
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        let url = post
            .select(&link_selector)
            .next()
            .and_then(|anchor| anchor.value().attr("href"))
            .map(str::to_string);
        let body = post
            .select(&body_selector)
            .next()
            .map(text_of)
            .unwrap_or_default();
        let published_at = post
            .select(&time_selector)
            .next()
            .and_then(|element| element.value().attr("datetime"))
            .and_then(timestamp);

        out.push(MoodleAnnouncement {
            id: url.as_deref().and_then(discussion_id),
            title,
            body,
            url,
            published_at,
        });
    }

    if !out.is_empty() {
        return Ok(out);
    }

    let row_selector = selector("tr.discussion, .discussion");
    let subject_selector = selector("a[href*=\"discuss.php?d=\"]");

    for row in doc.select(&row_selector) {
        let Some(anchor) = row.select(&subject_selector).next() else {
            continue;
        };
        let href = anchor.value().attr("href").unwrap_or("");
        let Some(id) = discussion_id(href) else {
            continue;
        };
        if out.iter().any(|item| item.id == Some(id)) {
            continue;
        }

        let title = text_of(anchor);
        if title.is_empty() {
            continue;
        }

        let published_at = row
            .select(&time_selector)
            .next()
            .and_then(|element| element.value().attr("datetime"))
            .and_then(timestamp);

        out.push(MoodleAnnouncement {
            id: Some(id),
            title,
            body: String::new(),
            url: Some(href.to_string()),
            published_at,
        });
    }

    Ok(out)
}

fn number(text: &str) -> Option<f64> {
    let cleaned = text.replace('\u{a0}', " ");
    let regex = Regex::new(r"-?\d+(?:[.,]\d+)?").expect("regex válida");
    let found = regex.find(&cleaned)?.as_str();
    found.replace(',', ".").parse().ok()
}

fn range_max(text: &str) -> Option<f64> {
    let cleaned = text.replace('\u{a0}', " ");
    let regex = Regex::new(r"[–—-]\s*(-?\d+(?:[.,]\d+)?)\s*$").expect("regex válida");
    let found = regex.captures(&cleaned)?.get(1)?.as_str();
    found.replace(',', ".").parse().ok()
}

pub fn parse_grade_total_html(html: &str) -> Result<MoodleGradeTotal, AppError> {
    let doc = Html::parse_document(html);
    let table = doc
        .select(&selector("table#user-grade"))
        .next()
        .or_else(|| doc.select(&selector("table.user-grade")).next())
        .ok_or_else(|| AppError::Parse("sin informe de notas".into()))?;

    let row_selector = selector("tr");
    let name_selector = selector(".column-itemname");
    let grade_selector = selector(".column-grade");
    let range_selector = selector(".column-range");
    let cell_selector = selector("th, td");

    let mut total = None;
    let mut max_total = None;

    for row in table.select(&row_selector) {
        let class = row.value().attr("class").unwrap_or("");
        let name = row
            .select(&name_selector)
            .next()
            .map(text_of)
            .unwrap_or_default();

        let is_total = class
            .split_whitespace()
            .any(|token| matches!(token, "course-total" | "total" | "course"))
            || name.to_lowercase().starts_with("total");
        if !is_total {
            continue;
        }

        let grade_text = row
            .select(&grade_selector)
            .next()
            .map(text_of)
            .or_else(|| {
                let cells: Vec<String> = row.select(&cell_selector).map(text_of).collect();
                cells.get(1).cloned()
            })
            .unwrap_or_default();

        total = number(&grade_text);
        max_total = row
            .select(&range_selector)
            .next()
            .map(text_of)
            .and_then(|range| range_max(&range));
    }

    total
        .map(|total| MoodleGradeTotal { total, max_total })
        .ok_or_else(|| AppError::Parse("sin total del curso".into()))
}

fn parse_activity_dates(doc: &Html) -> (Option<i64>, Option<i64>) {
    let block = doc
        .select(&selector("[data-region=\"activity-dates\"]"))
        .next()
        .or_else(|| doc.select(&selector(".activity-dates")).next());

    let Some(block) = block else {
        return (None, None);
    };
    let text = text_of(block);
    if text.is_empty() {
        return (None, None);
    }

    let due_at = labelled_date(&text, "Cierre");
    let cutoff_at = labelled_date(&text, "Corte");
    (due_at, cutoff_at)
}

fn labelled_date(text: &str, label: &str) -> Option<i64> {
    let pattern = format!(
        r"(?i){label}:\s*(?:[a-záéíóúñ]+,\s*)?(\d{{1,2}}) de ([a-záéíóúñ]+) de (\d{{4}}),?\s*(\d{{1,2}}):(\d{{2}})"
    );
    let regex = Regex::new(&pattern).ok()?;
    let caps = regex.captures(text)?;

    let day: u32 = caps.get(1)?.as_str().parse().ok()?;
    let month_name = caps.get(2)?.as_str().to_lowercase();
    let month = MONTHS
        .iter()
        .find(|(name, _)| *name == month_name)
        .map(|(_, number)| *number)?;
    let year: i32 = caps.get(3)?.as_str().parse().ok()?;
    let hour: u32 = caps.get(4)?.as_str().parse().ok()?;
    let minute: u32 = caps.get(5)?.as_str().parse().ok()?;

    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, 0)?;
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|date| date.timestamp())
}

pub fn parse_assignment_html(html: &str) -> Result<MoodleAssignment, AppError> {
    if html.trim().is_empty() {
        return Err(AppError::Parse("HTML vacío".into()));
    }

    let doc = Html::parse_document(html);
    let container = doc
        .select(&selector(".submissionstatustable"))
        .next()
        .ok_or_else(|| AppError::Parse("sin estado de entrega".into()))?;

    let row_selector = selector("tr");
    let cell_selector = selector("th, td");
    let mut submission = None;
    let mut grading = None;

    for row in container.select(&row_selector) {
        let cells: Vec<String> = row.select(&cell_selector).map(text_of).collect();
        if cells.len() < 2 {
            continue;
        }
        if cells[0].contains("Estado de la entrega") {
            submission = Some(cells[1].clone());
        } else if cells[0].contains("Estado de la calificación") {
            grading = Some(cells[1].clone());
        }
    }

    let status_raw =
        submission.ok_or_else(|| AppError::Parse("sin estado de la entrega".into()))?;
    let lower = status_raw.to_lowercase();
    let submitted =
        if lower.contains("no se han realizado envíos") || lower.contains("no se ha realizado") {
            false
        } else if lower.contains("enviado") || lower.contains("entregado") {
            true
        } else {
            return Err(AppError::Parse(format!(
                "estado de entrega desconocido: {status_raw}"
            )));
        };

    let grading = grading.unwrap_or_default();
    let graded = !grading.is_empty() && !grading.to_lowercase().contains("sin calificar");

    let (due_at, cutoff_at) = parse_activity_dates(&doc);

    Ok(MoodleAssignment {
        submitted,
        graded,
        status_raw,
        due_at,
        cutoff_at,
    })
}

/// Attachments of an activity page. The ones from the theme are discarded
/// (`/theme_*/favicon`), since they appear on every page.
pub fn parse_attachments(html: &str) -> Vec<MoodleFile> {
    use std::collections::HashSet;

    let doc = Html::parse_document(html);
    let link = selector("a[href*=\"pluginfile.php\"]");
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for anchor in doc.select(&link) {
        let href = anchor.value().attr("href").unwrap_or("");
        if href.contains("/theme_") || !seen.insert(href.to_string()) {
            continue;
        }

        let mut name = text_of(anchor);
        if name.is_empty() {
            name = href
                .split('/')
                .rfind(|part| !part.is_empty())
                .unwrap_or("archivo")
                .to_string();
        }

        out.push(MoodleFile {
            name,
            url: href.to_string(),
            mimetype: anchor.value().attr("data-mimetype").map(str::to_string),
            size: anchor
                .value()
                .attr("data-size")
                .and_then(|value| value.parse().ok()),
        });
    }

    out
}

pub fn parse_description(html: &str) -> String {
    let doc = Html::parse_document(html);
    for css in [
        "#intro",
        "[data-region=\"activity-description\"]",
        ".activity-description",
    ] {
        if let Some(element) = doc.select(&selector(css)).next() {
            let text = text_of(element);
            if !text.is_empty() {
                return text;
            }
        }
    }
    String::new()
}
