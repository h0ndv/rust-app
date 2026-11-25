use chrono::DateTime;
use regex::Regex;
use serde_json::Value;

use crate::ellucian::models::{AcademicPeriod, EllucianClass, InstructionalEvent};
use crate::error::AppError;

fn payload(raw: &str) -> Result<Value, AppError> {
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|e| AppError::Parse(format!("Ellucian did not return JSON: {e}")))?;

    if let Some(errors) = parsed.get("errors").and_then(Value::as_array)
        && let Some(first) = errors.first()
    {
        let code = first.get("code").and_then(Value::as_str).unwrap_or("error");
        let message = first.get("message").and_then(Value::as_str).unwrap_or("");
        return Err(AppError::Parse(format!(
            "Ellucian returned {code}: {message}"
        )));
    }

    Ok(parsed)
}

fn edges<'a>(payload: &'a Value, field: &str) -> Result<&'a [Value], AppError> {
    payload
        .pointer(&format!("/data/{field}/edges"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| AppError::Parse(format!("missing {field}")))
}

fn clock(value: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|date| date.format("%H:%M").to_string())
}

fn weekday(name: &str) -> Option<i64> {
    match name.to_lowercase().as_str() {
        "monday" => Some(1),
        "tuesday" => Some(2),
        "wednesday" => Some(3),
        "thursday" => Some(4),
        "friday" => Some(5),
        "saturday" => Some(6),
        "sunday" => Some(7),
        _ => None,
    }
}

fn text_at<'a>(value: &'a Value, path: &str) -> Option<&'a str> {
    value.pointer(path).and_then(Value::as_str)
}

fn instructor(node: &Value) -> Option<String> {
    let first = text_at(node, "/instructorsInfo/firstName")
        .unwrap_or_default()
        .trim();
    let last = text_at(node, "/instructorsInfo/lastName")
        .unwrap_or_default()
        .trim();

    if first.is_empty() && last.is_empty() {
        return None;
    }

    let full = format!("{first} {}", last.replace(';', " "))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!full.is_empty()).then_some(full)
}

/// One response per section. The same block may repeat on several days,
/// and in that case each day is a separate row of the schedule.
pub fn parse_schedule_response(raw: &str) -> Result<Vec<InstructionalEvent>, AppError> {
    let payload = payload(raw)?;
    let edges = edges(&payload, "instructionalEvents")?;

    let mut out = Vec::new();
    for edge in edges {
        let node = edge
            .get("node")
            .ok_or_else(|| AppError::Parse("event without a node".into()))?;

        let id = text_at(node, "/id").unwrap_or_default().to_string();
        let start_on = text_at(node, "/recurrence/timePeriod/startOn").unwrap_or_default();
        let end_on = text_at(node, "/recurrence/timePeriod/endOn").unwrap_or_default();

        let (Some(start_time), Some(end_time)) = (clock(start_on), clock(end_on)) else {
            continue;
        };

        let days: Vec<String> = node
            .pointer("/recurrence/repeatRule/daysOfWeek")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        let room = text_at(node, "/locations/0/location/room/title").map(str::to_string);
        let building =
            text_at(node, "/locations/0/location/room/building/title").map(str::to_string);
        let instructor = instructor(node);
        let repeated = days.len() > 1;

        for day in days {
            let Some(day_of_week) = weekday(&day) else {
                continue;
            };
            let event_id = if repeated {
                format!("{id}#{day}")
            } else {
                id.clone()
            };
            out.push(InstructionalEvent {
                id: event_id,
                section_id: None,
                title: None,
                term: None,
                day_of_week,
                start_time: start_time.clone(),
                end_time: end_time.clone(),
                room: room.clone(),
                building: building.clone(),
                instructor: instructor.clone(),
            });
        }
    }

    Ok(out)
}

fn course_title(section: &Value) -> String {
    section
        .pointer("/course/titles")
        .and_then(Value::as_array)
        .map(|titles| {
            titles
                .iter()
                .filter_map(|title| title.get("value").and_then(Value::as_str))
                .max_by_key(|value| value.len())
                .unwrap_or_default()
                .to_string()
        })
        .unwrap_or_default()
}

/// Looks for a Moodle shortname (`30061.202620`) in any field of the
/// section. The extension does not always carry the AULA VIRTUAL link.
pub fn moodle_shortname(node: &Value) -> Option<String> {
    let regex = Regex::new(r"\b\d{5}\.\d{6}\b").ok()?;

    fn walk(value: &Value, regex: &Regex) -> Option<String> {
        match value {
            Value::String(text) => regex.find(text).map(|found| found.as_str().to_string()),
            Value::Array(items) => items.iter().find_map(|item| walk(item, regex)),
            Value::Object(map) => map.values().find_map(|item| walk(item, regex)),
            _ => None,
        }
    }

    walk(node, &regex)
}

pub fn parse_classes_response(raw: &str) -> Result<Vec<EllucianClass>, AppError> {
    let payload = payload(raw)?;
    let edges = edges(&payload, "sectionRegistrations")?;

    let mut out = Vec::new();
    for edge in edges {
        let Some(section) = edge.pointer("/node/section") else {
            continue;
        };
        let Some(section_id) = text_at(section, "/id") else {
            continue;
        };

        out.push(EllucianClass {
            section_id: section_id.to_string(),
            crn: text_at(section, "/code").unwrap_or_default().to_string(),
            subject: text_at(section, "/course/subject/abbreviation")
                .unwrap_or_default()
                .to_string(),
            number: text_at(section, "/course/number")
                .unwrap_or_default()
                .to_string(),
            title: course_title(section),
            term: text_at(section, "/reportingAcademicPeriod/code")
                .unwrap_or_default()
                .to_string(),
            term_title: text_at(section, "/reportingAcademicPeriod/title")
                .unwrap_or_default()
                .to_string(),
            moodle_shortname: moodle_shortname(section),
        });
    }

    Ok(out)
}

pub fn parse_periods_response(raw: &str) -> Result<Vec<AcademicPeriod>, AppError> {
    let payload = payload(raw)?;
    let edges = edges(&payload, "sectionRegistrations")?;

    let mut out: Vec<AcademicPeriod> = Vec::new();
    for edge in edges {
        let Some(period) = edge.pointer("/node/section/reportingAcademicPeriod") else {
            continue;
        };
        let Some(id) = text_at(period, "/id") else {
            continue;
        };
        if out.iter().any(|existing| existing.id == id) {
            continue;
        }

        out.push(AcademicPeriod {
            id: id.to_string(),
            title: text_at(period, "/title").unwrap_or_default().to_string(),
            start_on: text_at(period, "/startOn")
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .map(|date| date.timestamp()),
        });
    }

    Ok(out)
}

pub fn current_period(periods: &[AcademicPeriod], now: i64) -> Option<&AcademicPeriod> {
    let started = periods
        .iter()
        .filter(|period| period.start_on.is_some_and(|start| start <= now))
        .max_by_key(|period| period.start_on);

    started.or_else(|| {
        periods
            .iter()
            .filter(|period| period.start_on.is_some())
            .min_by_key(|period| period.start_on)
    })
}
