use contracts::{DashboardDto, ResourceDto, ScheduleEventDto, TaskDto};
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::format::format_date_local;
use crate::i18n::{Language, resolve_language, t};
use crate::router::Route;
use crate::state::use_app;

fn is_empty(dashboard: &DashboardDto) -> bool {
    dashboard.overdue.is_empty()
        && dashboard.due_soon.is_empty()
        && dashboard.next_classes.is_empty()
        && dashboard.new_resources.is_empty()
}

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

#[component]
fn TaskList(tasks: Vec<TaskDto>) -> Element {
    let language = active_language();

    rsx! {
        ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
            for task in tasks {
                li { key: "{task.id}", class: "flex flex-col gap-1 p-3",
                    span { class: "font-medium", "{task.title}" }
                    span { class: "text-xs text-muted-foreground",
                        "{task.course_name} · {format_date_local(task.due_at, language)}"
                    }
                }
            }
        }
    }
}

#[component]
fn ClassList(events: Vec<ScheduleEventDto>) -> Element {
    rsx! {
        ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
            for event in events {
                li { key: "{event.id}", class: "flex flex-wrap items-center gap-2 p-3 text-sm",
                    span { class: "font-medium", "{event.title}" }
                    span { class: "text-muted-foreground",
                        "{event.start.as_deref().unwrap_or_default()} - {event.end.as_deref().unwrap_or_default()}"
                    }
                    if let Some(room) = event.room {
                        span { class: "text-muted-foreground", "{room}" }
                    }
                }
            }
        }
    }
}

#[component]
fn ResourceList(resources: Vec<ResourceDto>) -> Element {
    rsx! {
        ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
            for resource in resources {
                li { key: "{resource.id}", class: "p-3 text-sm", "{resource.title}" }
            }
        }
    }
}

#[component]
pub fn HomeSections(dashboard: DashboardDto) -> Element {
    if is_empty(&dashboard) {
        return rsx! {
            EmptyNote { "{t(\"home.empty\")}" }
        };
    }

    let has_overdue = !dashboard.overdue.is_empty();
    let has_due_soon = !dashboard.due_soon.is_empty();
    let has_classes = !dashboard.next_classes.is_empty();
    let has_resources = !dashboard.new_resources.is_empty();

    rsx! {
        div { class: "flex flex-col gap-8",
            if has_overdue {
                section { class: "flex flex-col gap-2",
                    h3 { class: "text-sm font-semibold", "{t(\"home.overdue\")}" }
                    TaskList { tasks: dashboard.overdue }
                }
            }
            if has_due_soon {
                section { class: "flex flex-col gap-2",
                    h3 { class: "text-sm font-semibold", "{t(\"home.dueSoon\")}" }
                    TaskList { tasks: dashboard.due_soon }
                }
            }
            if has_classes {
                section { class: "flex flex-col gap-2",
                    h3 { class: "text-sm font-semibold", "{t(\"home.nextClasses\")}" }
                    ClassList { events: dashboard.next_classes }
                }
            }
            if has_resources {
                section { class: "flex flex-col gap-2",
                    h3 { class: "text-sm font-semibold", "{t(\"home.newResources\")}" }
                    ResourceList { resources: dashboard.new_resources }
                }
            }
        }
    }
}

/// The dashboard plus the link to the full task list. It owns the `Link`
/// because the router context only exists outside the SSR tests.
#[component]
pub fn HomeView(dashboard: DashboardDto) -> Element {
    let empty = is_empty(&dashboard);

    rsx! {
        div { class: "flex flex-col gap-8",
            HomeSections { dashboard }
            if !empty {
                Link {
                    to: Route::Tasks {},
                    class: "text-sm text-primary underline-offset-4 hover:underline",
                    "{t(\"home.allTasks\")}"
                }
            }
        }
    }
}

#[component]
pub fn HomeScreen() -> Element {
    let app = use_app();
    let api = app.api.clone();
    let data_version = app.data_version;
    let dashboard = use_resource(move || {
        let _ = data_version();
        let api = api.clone();
        async move { api.get_dashboard().await }
    });

    let body = match dashboard.cloned() {
        Some(Ok(dashboard)) => rsx! { HomeView { dashboard } },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("home.title"), subtitle: t("home.subtitle"),
            {body}
        }
    }
}
