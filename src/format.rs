use chrono::{DateTime, FixedOffset, Locale, NaiveDate, Timelike};

use crate::i18n::{Language, t};

const WEEK: &str = "%A";

fn chrono_locale(locale: Language) -> Locale {
    match locale {
        Language::En => Locale::en_US,
        Language::Es => Locale::es_CL,
    }
}

/// Mirrors `Intl.DateTimeFormat` with day, short month, hour and minute:
/// English keeps the month first and Spanish keeps the day first, both in
/// 12-hour time. Spanish omits `%p` because chrono's es_CL data has no AM/PM
/// strings; `format_date` appends the marker from the local hour.
fn date_pattern(locale: Language) -> &'static str {
    match locale {
        Language::En => "%b %d, %I:%M %p",
        Language::Es => "%d-%b, %I:%M",
    }
}

/// Formats an epoch second at a fixed offset east of UTC (Chile is `-180` in
/// summer). Missing or unrepresentable values fall back to the catalog copy.
pub fn format_date(epoch: Option<i64>, locale: Language, offset_minutes: i32) -> String {
    let Some(epoch) = epoch else {
        return t("common.noDate");
    };
    let Some(offset) = offset_minutes
        .checked_mul(60)
        .and_then(FixedOffset::east_opt)
    else {
        return t("common.noDate");
    };
    let Some(utc) = DateTime::from_timestamp(epoch, 0) else {
        return t("common.noDate");
    };

    let local = utc.with_timezone(&offset);
    let formatted = local
        .format_localized(date_pattern(locale), chrono_locale(locale))
        .to_string();

    match locale {
        Language::En => formatted,
        Language::Es => {
            let marker = if local.hour() < 12 { "a. m." } else { "p. m." };
            format!("{formatted} {marker}")
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn format_date_local(epoch: Option<i64>, locale: Language) -> String {
    // The offset belongs to the instant being formatted, not to today.
    let offset = epoch.map_or(0, |epoch| {
        let date = js_sys::Date::new(&js_sys::wasm_bindgen::JsValue::from_f64(
            epoch as f64 * 1000.0,
        ));
        -(date.get_timezone_offset() as i32)
    });
    format_date(epoch, locale, offset)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn format_date_local(epoch: Option<i64>, locale: Language) -> String {
    format_date(epoch, locale, 0)
}

pub fn format_grade(value: Option<f64>, locale: Language) -> String {
    let Some(value) = value else {
        return t("common.notPublished");
    };

    separate_decimals(format!("{:.1}", round_one_decimal(value)), locale)
}

pub fn format_size(bytes: Option<i64>, locale: Language) -> String {
    let Some(bytes) = bytes.filter(|bytes| *bytes != 0) else {
        return String::new();
    };

    let (value, unit) = if bytes < 1024 {
        (bytes as f64, "B")
    } else if bytes < 1024 * 1024 {
        (bytes as f64 / 1024.0, "KB")
    } else {
        (bytes as f64 / (1024.0 * 1024.0), "MB")
    };

    format!(
        "{} {unit}",
        separate_decimals(trim_zeros(round_one_decimal(value)), locale)
    )
}

pub fn weekdays(locale: Language) -> [String; 7] {
    // 2024-01-01 was a Monday, so the array keeps the UI order.
    let monday = NaiveDate::from_ymd_opt(2024, 1, 1).expect("a valid reference date");

    std::array::from_fn(|index| {
        let day = monday + chrono::Duration::days(index as i64);
        day.format_localized(WEEK, chrono_locale(locale))
            .to_string()
    })
}

/// `Intl.NumberFormat` rounds half away from zero; Rust's formatter rounds
/// half to even, so grades such as 6.25 would otherwise become 6.2.
fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn trim_zeros(value: f64) -> String {
    let text = format!("{value:.1}");
    match text.strip_suffix(".0") {
        Some(whole) => whole.to_string(),
        None => text,
    }
}

fn separate_decimals(value: String, locale: Language) -> String {
    match locale {
        Language::En => value,
        Language::Es => value.replace('.', ","),
    }
}
