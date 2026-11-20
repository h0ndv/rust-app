use serde_json::Value;

use crate::banner::models::{BannerGrade, BannerOption, TermGpa};
use crate::error::AppError;

/// Banner answers with the SSO page when the session does not work for the
/// endpoint. It is HTML, never JSON.
pub fn looks_like_sso(raw: &str) -> bool {
    let head: String = raw.chars().take(2000).collect();
    head.contains("<html")
        || head.contains("<!DOCTYPE")
        || raw.contains("BssoInterrupt")
        || raw.contains("name=\"PageID\"")
}

fn parse_json(raw: &str) -> Result<Value, AppError> {
    if looks_like_sso(raw) {
        return Err(AppError::Unauthorized);
    }
    serde_json::from_str(raw).map_err(|e| AppError::Parse(format!("Banner no devolvió JSON: {e}")))
}

fn text(row: &Value, key: &str) -> String {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn option(row: &Value, key: &str) -> Option<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.is_empty())
}

pub fn parse_courses_response(raw: &str) -> Result<Vec<BannerGrade>, AppError> {
    let payload = parse_json(raw)?;

    if payload.get("success").and_then(Value::as_bool) == Some(false) {
        return Err(AppError::Parse(
            "Banner rechazó la consulta de notas".into(),
        ));
    }

    let rows = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Parse("Banner no devolvió notas".into()))?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(BannerGrade {
            subject: text(row, "subjectCode"),
            number: text(row, "courseNumber"),
            crn: text(row, "courseReferenceNumber"),
            section: text(row, "section"),
            title: text(row, "courseTitle"),
            final_grade: option(row, "finalGrade"),
            midterm_grade: option(row, "midtermGrade"),
            history_final_grade: option(row, "historyFinalGrade"),
            calculated_final_grade: option(row, "calculatedFinalGrade"),
            term: text(row, "termCode"),
            term_title: text(row, "termDescription"),
            level: text(row, "levelCode"),
        });
    }
    Ok(out)
}

pub fn parse_options_response(raw: &str) -> Result<Vec<BannerOption>, AppError> {
    let payload = parse_json(raw)?;
    let rows = payload
        .as_array()
        .ok_or_else(|| AppError::Parse("Banner no devolvió una lista".into()))?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let code = text(row, "code");
        if code.is_empty() {
            continue;
        }
        out.push(BannerOption {
            code,
            description: text(row, "description"),
        });
    }
    Ok(out)
}

fn as_number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().replace(',', ".").parse().ok(),
        _ => None,
    }
}

pub fn parse_gpa_response(raw: &str) -> Result<TermGpa, AppError> {
    let payload = parse_json(raw)?;

    Ok(TermGpa {
        selected_term: as_number(payload.pointer("/selectedTerm/gpa")),
        cumulative: as_number(payload.pointer("/cumulative/gpa")),
    })
}
