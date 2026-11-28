use contracts::{TaskDto, TaskFilter};
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::domain::{TaskStatus, task_status};
use crate::format::{format_date_local, format_grade};
use crate::i18n::{Language, resolve_language, t};
use crate::open::open_in_app;
use crate::state::{spawn_future, use_app};

const GROUPS: [(TaskStatus, &str); 7] = [
    (TaskStatus::Overdue, "tasks.groups.overdue"),
    (TaskStatus::Pending, "tasks.groups.pending"),
    (TaskStatus::Submitted, "tasks.groups.submitted"),
    (TaskStatus::Graded, "tasks.groups.graded"),
    (TaskStatus::NoData, "tasks.groups.no_data"),
    (TaskStatus::Unconfirmed, "tasks.groups.unconfirmed"),
    (TaskStatus::Informational, "tasks.groups.informational"),
];

const FILTERS: [(TaskFilter, &str); 8] = [
    (TaskFilter::All, "tasks.filters.all"),
    (TaskFilter::Overdue, "tasks.filters.overdue"),
    (TaskFilter::Pending, "tasks.filters.pending"),
    (TaskFilter::Submitted, "tasks.filters.submitted"),
    (TaskFilter::Graded, "tasks.filters.graded"),
    (TaskFilter::NoData, "tasks.filters.no_data"),
    (TaskFilter::Unconfirmed, "tasks.filters.unconfirmed"),
    (TaskFilter::Informational, "tasks.filters.informational"),
];

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

#[component]
fn TaskRow(task: TaskDto, language: Language, on_open: EventHandler<String>) -> Element {
    let due = format_date_local(task.due_at, language);
    let url = task.url.clone();

    rsx! {
        li { class: "flex flex-wrap items-center gap-3 p-3",
            div { class: "flex min-w-0 flex-1 flex-col",
                span { class: "flex min-w-0 items-center gap-2",
                    span { class: "truncate font-medium", "{task.title}" }
                    if task.is_new {
                        span { class: "rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary",
                            "{t(\"common.new\")}"
                        }
                    }
                }
                span { class: "truncate text-xs text-muted-foreground", "{task.course_name}" }
            }
            span { class: "text-xs text-muted-foreground", "{due}" }
            if let Some(grade) = task.grade {
                span { class: "text-sm font-medium", "{format_grade(Some(grade), language)}" }
            }
            if let Some(url) = url {
                button {
                    r#type: "button",
                    class: "text-xs text-primary underline-offset-4 hover:underline",
                    onclick: move |_| on_open.call(url.clone()),
                    "{t(\"common.open\")}"
                }
            }
        }
    }
}

#[component]
pub fn TasksView(tasks: Vec<TaskDto>, on_open: EventHandler<String>) -> Element {
    let language = active_language();

    if tasks.is_empty() {
        return rsx! {
            EmptyNote { "{t(\"tasks.empty\")}" }
        };
    }

    // Statuses the domain does not recognize have no group, as in React.
    let mut buckets: [Vec<TaskDto>; 7] = std::array::from_fn(|_| Vec::new());
    for task in tasks {
        if let Some(status) = task_status(task.status.as_deref()) {
            buckets[status as usize].push(task);
        }
    }

    let groups: Vec<(TaskStatus, &str, Vec<TaskDto>)> = GROUPS
        .into_iter()
        .zip(buckets)
        .filter_map(|((status, key), items)| (!items.is_empty()).then_some((status, key, items)))
        .collect();

    rsx! {
        div { class: "flex flex-col gap-6",
            for (status, key, items) in groups {
                section { key: "{status:?}", class: "flex flex-col gap-2",
                    h3 { class: "text-xs font-semibold tracking-wide text-muted-foreground uppercase",
                        "{t(key)}"
                    }
                    ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
                        for task in items {
                            TaskRow { key: "{task.id}", task, language, on_open }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn TasksScreen() -> Element {
    let app = use_app();
    let resource_api = app.api.clone();
    let data_version = app.data_version;
    let mut filter = use_signal(|| TaskFilter::All);

    let tasks = use_resource(move || {
        let _ = data_version();
        let selected = filter();
        let api = resource_api.clone();
        async move { api.list_tasks(selected).await }
    });

    let active = filter();

    let actions = rsx! {
        div { class: "flex flex-wrap gap-2",
            for (value, key) in FILTERS {
                button {
                    key: "{key}",
                    r#type: "button",
                    class: if value == active {
                        "rounded-md bg-primary px-3 py-1 text-sm text-primary-foreground"
                    } else {
                        "rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                    },
                    onclick: move |_| filter.set(value),
                    "{t(key)}"
                }
            }
        }
    };

    let on_open = {
        let api = app.api.clone();
        let toasts = app.toasts.clone();
        move |url: String| {
            let api = api.clone();
            let toasts = toasts.clone();
            spawn_future(async move {
                open_in_app(&api, &toasts, &url).await;
            });
        }
    };

    let body = match tasks.cloned() {
        Some(Ok(tasks)) => rsx! { TasksView { tasks, on_open } },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("tasks.title"), subtitle: t("tasks.subtitle"), actions,
            {body}
        }
    }
}
