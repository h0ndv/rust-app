use contracts::ScheduleEventDto;
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::format::weekdays;
use crate::i18n::{Language, resolve_language, t};
use crate::state::use_app;

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

struct BlockView {
    id: String,
    title: String,
    time: String,
    room: String,
    instructor: Option<String>,
}

struct DayView {
    number: usize,
    label: String,
    blocks: Vec<BlockView>,
}

/// Resolves the catalog copy and the per-day ordering before RSX so the markup
/// only displays strings. Days outside 1..=7 have no column, as in React.
fn day_views(events: Vec<ScheduleEventDto>, language: Language) -> Vec<DayView> {
    let mut by_day: [Vec<ScheduleEventDto>; 7] = std::array::from_fn(|_| Vec::new());
    for event in events {
        if let Some(day) = event.day.filter(|day| (1..=7).contains(day)) {
            by_day[(day - 1) as usize].push(event);
        }
    }

    let class_fallback = t("schedule.classFallback");
    let no_room = t("schedule.noRoom");

    weekdays(language)
        .into_iter()
        .zip(by_day)
        .enumerate()
        .map(|(index, (label, mut blocks))| {
            blocks.sort_by(|a, b| {
                a.start
                    .as_deref()
                    .unwrap_or_default()
                    .cmp(b.start.as_deref().unwrap_or_default())
            });

            let blocks = blocks
                .into_iter()
                .map(|block| BlockView {
                    id: block.id,
                    title: if block.title.is_empty() {
                        class_fallback.clone()
                    } else {
                        block.title
                    },
                    time: format!(
                        "{} - {}",
                        block.start.as_deref().unwrap_or_default(),
                        block.end.as_deref().unwrap_or_default()
                    ),
                    room: block.room.unwrap_or_else(|| no_room.clone()),
                    instructor: block.instructor,
                })
                .collect();

            DayView {
                number: index + 1,
                label,
                blocks,
            }
        })
        .collect()
}

#[component]
pub fn ScheduleView(events: Vec<ScheduleEventDto>) -> Element {
    if events.is_empty() {
        return rsx! {
            EmptyNote { "{t(\"schedule.empty\")}" }
        };
    }

    let days = day_views(events, active_language());

    rsx! {
        div { class: "grid gap-3 md:grid-cols-2 xl:grid-cols-7",
            for day in days {
                section {
                    key: "{day.number}",
                    class: "flex min-h-32 flex-col gap-2 rounded-lg border border-border p-3",
                    h3 { class: "text-xs font-semibold tracking-wide text-muted-foreground uppercase",
                        "{day.label}"
                    }
                    if day.blocks.is_empty() {
                        span { class: "text-xs text-muted-foreground",
                            "{t(\"schedule.noClasses\")}"
                        }
                    } else {
                        for block in day.blocks {
                            article {
                                key: "{block.id}",
                                class: "flex flex-col gap-1 rounded-md bg-muted/50 p-2 text-sm",
                                span { class: "leading-tight font-medium", "{block.title}" }
                                span { class: "text-xs text-muted-foreground", "{block.time}" }
                                span { class: "text-xs text-muted-foreground", "{block.room}" }
                                if let Some(instructor) = block.instructor {
                                    span { class: "text-xs text-muted-foreground", "{instructor}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ScheduleScreen() -> Element {
    let app = use_app();
    let api = app.api.clone();
    let data_version = app.data_version;

    let schedule = use_resource(move || {
        let _ = data_version();
        let api = api.clone();
        async move { api.get_schedule().await }
    });

    let body = match schedule.cloned() {
        Some(Ok(events)) => rsx! { ScheduleView { events } },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("schedule.title"), subtitle: t("schedule.subtitle"),
            {body}
        }
    }
}
