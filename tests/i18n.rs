#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::BTreeSet;

use rust_app_ui::i18n::{LANGUAGES, Language, resolve_language, t, tn};
use serde_yaml::Value;

fn keys(path: &str) -> BTreeSet<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path}: {error}"));
    let root: Value = serde_yaml::from_str(&text).unwrap_or_else(|error| panic!("{path}: {error}"));

    let version = root.get("_version").and_then(Value::as_u64).unwrap_or(1);
    let mut keys = BTreeSet::new();
    collect(&root, "", version, &mut keys);
    keys
}

fn collect(value: &Value, prefix: &str, version: u64, keys: &mut BTreeSet<String>) {
    let Value::Mapping(mapping) = value else {
        return;
    };

    for (key, child) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };
        if prefix.is_empty() && key == "_version" {
            continue;
        }

        let path = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        };

        match child {
            // Version 2 stores the locale on the leaf; the key is the path itself.
            Value::Mapping(_) if version == 2 && prefix.is_empty() => {
                keys.insert(path);
            }
            Value::String(_) => {
                keys.insert(path);
            }
            Value::Mapping(_) => collect(child, &path, version, keys),
            _ => {}
        }
    }
}

#[test]
fn both_catalogs_have_the_same_keys() {
    let english = keys("locales/en.yml");
    let spanish = keys("locales/es.yml");

    assert_eq!(english, spanish);
    assert_eq!(english.len(), 207, "the catalogs carry 207 leaves");
}

#[test]
fn the_catalogs_keep_no_handlebars_placeholders() {
    for path in ["locales/en.yml", "locales/es.yml"] {
        let text = std::fs::read_to_string(path).unwrap();
        assert!(
            !text.contains("{{"),
            "{path} still has a handlebars placeholder"
        );
    }
}

#[test]
fn translates_both_languages() {
    common::with_locale(Language::Es, || assert_eq!(t("nav.courses"), "Ramos"));
    common::with_locale(Language::En, || assert_eq!(t("nav.courses"), "Courses"));
}

#[test]
fn translates_dynamic_keys() {
    let key = "nav.schedule";
    common::with_locale(Language::Es, || assert_eq!(t(key), "Horario"));
}

#[test]
fn plural_uses_the_count() {
    common::with_locale(Language::En, || {
        assert_eq!(tn("notifications.overdue", 1), "1 overdue task");
        assert_eq!(tn("notifications.overdue", 3), "3 overdue tasks");
    });
}

#[test]
fn spanish_plural_uses_the_count() {
    common::with_locale(Language::Es, || {
        assert_eq!(tn("notifications.overdue", 1), "1 atrasada");
        assert_eq!(tn("notifications.overdue", 3), "3 atrasadas");
    });
}

#[test]
fn resolves_unknown_languages_to_english() {
    assert_eq!(resolve_language("es"), Language::Es);
    assert_eq!(resolve_language("en"), Language::En);
    assert_eq!(resolve_language("fr"), Language::En);
    assert_eq!(resolve_language(""), Language::En);
    assert_eq!(LANGUAGES, [Language::En, Language::Es]);
}

#[test]
fn language_codes_match_the_catalogs() {
    assert_eq!(Language::En.code(), "en");
    assert_eq!(Language::Es.code(), "es");
    assert_eq!(Language::Es.locale(), "es-CL");
}
