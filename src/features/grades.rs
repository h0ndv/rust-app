use contracts::GradesDto;
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::format::format_grade;
use crate::i18n::{Language, resolve_language, t};
use crate::state::use_app;

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

/// The two tables and the GPA line. It never averages Banner and Moodle.
#[component]
pub fn GradesView(grades: GradesDto) -> Element {
    let language = active_language();
    let GradesDto {
        banner,
        moodle_totals,
        gpa,
        ..
    } = grades;

    rsx! {
        div { class: "flex flex-col gap-8",
            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"grades.official\")}" }
                if banner.is_empty() {
                    EmptyNote { "{t(\"grades.notPublishedYet\")}" }
                } else {
                    table { class: "w-full text-sm",
                        thead {
                            tr { class: "border-b border-border text-left text-xs tracking-wide text-muted-foreground uppercase",
                                th { class: "py-2", "{t(\"grades.course\")}" }
                                th { class: "py-2 text-right", "{t(\"grades.grade\")}" }
                            }
                        }
                        tbody {
                            for row in banner {
                                tr { key: "{row.course}-{row.name}", class: "border-b border-border/60",
                                    td { class: "py-2", "{row.course}" }
                                    td { class: "py-2 text-right font-medium",
                                        "{format_grade(row.grade, language)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"grades.moodleTotal\")}" }
                if moodle_totals.is_empty() {
                    EmptyNote { "{t(\"grades.notPublishedYet\")}" }
                } else {
                    table { class: "w-full text-sm",
                        thead {
                            tr { class: "border-b border-border text-left text-xs tracking-wide text-muted-foreground uppercase",
                                th { class: "py-2", "{t(\"grades.course\")}" }
                                th { class: "py-2 text-right", "{t(\"grades.total\")}" }
                            }
                        }
                        tbody {
                            for row in moodle_totals {
                                tr { key: "{row.course}", class: "border-b border-border/60",
                                    td { class: "py-2", "{row.course}" }
                                    td { class: "py-2 text-right font-medium",
                                        "{format_grade(row.total, language)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(gpa) = gpa {
                p { class: "text-sm text-muted-foreground",
                    {rust_i18n::t!("grades.termGpa", value = format_grade(Some(gpa), language))
                        .to_string()}
                }
            }
        }
    }
}

#[component]
pub fn GradesScreen() -> Element {
    let app = use_app();
    let api = app.api.clone();
    let data_version = app.data_version;
    let mut term = use_signal(|| None::<String>);

    let grades = use_resource(move || {
        let _ = data_version();
        let selected = term();
        let api = api.clone();
        async move { api.list_grades(selected.as_deref()).await }
    });

    let state = grades.cloned();
    let actions = match &state {
        Some(Ok(data)) if data.terms.len() > 1 => {
            let selected = data.term.clone();
            let terms = data.terms.clone();

            Some(rsx! {
                select {
                    class: "rounded-md border border-border bg-background px-3 py-1 text-sm",
                    value: "{selected}",
                    onchange: move |event| term.set(Some(event.value())),
                    for item in terms {
                        option {
                            key: "{item}",
                            value: "{item}",
                            selected: item == selected,
                            "{item}"
                        }
                    }
                }
            })
        }
        _ => None,
    };

    let body = match state {
        Some(Ok(data)) => rsx! { GradesView { grades: data } },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("grades.title"), subtitle: t("grades.subtitle"), actions,
            {body}
        }
    }
}
