use contracts::{ActivityRow, CourseDetailDto, CourseSectionDto, FileRow, GradeRow, ScheduleRow};
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::domain::{TASK_STATUS_KEY, TaskStatus, task_status};
use crate::format::{format_date_local, format_grade};
use crate::i18n::{Language, resolve_language, t};
use crate::open::open_in_app;
use crate::state::{spawn_future, use_app};

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

#[component]
fn SectionRow(section: CourseSectionDto) -> Element {
    let label = section
        .section
        .unwrap_or_else(|| t("courseDetail.noSection"));
    let teacher = section
        .teacher
        .unwrap_or_else(|| t("courseDetail.noTeacher"));

    rsx! {
        li { class: "rounded-md border border-border px-3 py-1", "{label} · {teacher}" }
    }
}

#[component]
fn TaskRow(task: ActivityRow, focus: Option<String>, language: Language) -> Element {
    let focused = focus.as_deref() == Some(task.id.as_str());
    let class = if focused {
        "flex flex-wrap items-center gap-3 p-3 text-sm bg-primary/5"
    } else {
        "flex flex-wrap items-center gap-3 p-3 text-sm"
    };
    let status = task_status(task.status.as_deref()).unwrap_or(TaskStatus::NoData);
    let status = t(TASK_STATUS_KEY[status as usize]);
    let due = format_date_local(task.due_at, language);

    rsx! {
        li {
            id: "focus-{task.id}",
            aria_current: focused.then_some("true"),
            class,
            span { class: "min-w-0 flex-1 truncate", "{task.name}" }
            if task.acknowledged_at.is_none() {
                span { class: "rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary",
                    "{t(\"common.new\")}"
                }
            }
            span { class: "text-xs text-muted-foreground", "{status}" }
            span { class: "text-xs text-muted-foreground", "{due}" }
        }
    }
}

#[component]
fn ResourceRow(resource: FileRow, focus: Option<String>, on_open: EventHandler<String>) -> Element {
    let focused = focus.as_deref() == Some(resource.id.as_str());
    let class = if focused {
        "flex flex-wrap items-center gap-3 p-3 text-sm bg-primary/5"
    } else {
        "flex flex-wrap items-center gap-3 p-3 text-sm"
    };
    let url = resource.url.clone();

    rsx! {
        li {
            id: "focus-{resource.id}",
            aria_current: focused.then_some("true"),
            class,
            span { class: "min-w-0 flex-1 truncate", "{resource.name}" }
            button {
                r#type: "button",
                class: "text-xs text-primary underline-offset-4 hover:underline",
                onclick: move |_| on_open.call(url.clone()),
                "{t(\"common.open\")}"
            }
        }
    }
}

#[component]
fn GradeEntry(grade: GradeRow, language: Language) -> Element {
    rsx! {
        li { class: "flex items-center gap-3 p-3 text-sm",
            span { class: "min-w-0 flex-1 truncate", "{grade.name}" }
            span { class: "text-xs text-muted-foreground uppercase", "{grade.source}" }
            span { class: "font-medium", "{format_grade(grade.grade, language)}" }
        }
    }
}

#[component]
fn ScheduleEntry(block: ScheduleRow) -> Element {
    let title = if block.title.is_empty() {
        t("schedule.classFallback")
    } else {
        block.title
    };
    let start = block.start_time.unwrap_or_default();
    let end = block.end_time.unwrap_or_default();
    let room = block.room.unwrap_or_else(|| t("courseDetail.noRoom"));

    rsx! {
        li { class: "p-3 text-sm", "{title} · {start} - {end} · {room}" }
    }
}

#[component]
pub fn CourseDetailView(
    detail: CourseDetailDto,
    #[props(default)] focus: Option<String>,
    on_open: EventHandler<String>,
) -> Element {
    let language = active_language();

    // The effect tracks both values: rows that arrive with a new detail are
    // scrolled once they exist in the DOM.
    use_effect(use_reactive((&detail, &focus), |(_, focus)| {
        scroll_to_focus(focus.as_deref());
    }));

    let CourseDetailDto {
        group,
        tasks,
        resources,
        grades,
        schedule,
    } = detail;

    rsx! {
        div { class: "flex flex-col gap-8",
            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"courseDetail.sections\")}" }
                ul { class: "flex flex-wrap gap-2 text-sm",
                    for section in group.sections {
                        SectionRow { key: "{section.id}", section }
                    }
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"courseDetail.tasks\")}" }
                if tasks.is_empty() {
                    EmptyNote { "{t(\"courseDetail.noTasks\")}" }
                } else {
                    ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
                        for task in tasks {
                            TaskRow {
                                key: "{task.id}",
                                task,
                                focus: focus.clone(),
                                language,
                            }
                        }
                    }
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"courseDetail.resources\")}" }
                if resources.is_empty() {
                    EmptyNote { "{t(\"courseDetail.noResources\")}" }
                } else {
                    ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
                        for resource in resources {
                            ResourceRow {
                                key: "{resource.id}",
                                resource,
                                focus: focus.clone(),
                                on_open,
                            }
                        }
                    }
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"courseDetail.grades\")}" }
                if grades.is_empty() {
                    EmptyNote { "{t(\"courseDetail.noGrades\")}" }
                } else {
                    ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
                        for grade in grades {
                            GradeEntry { key: "{grade.id}", grade, language }
                        }
                    }
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-sm font-semibold", "{t(\"courseDetail.schedule\")}" }
                if schedule.is_empty() {
                    EmptyNote { "{t(\"courseDetail.noSchedule\")}" }
                } else {
                    ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
                        for block in schedule {
                            ScheduleEntry { key: "{block.id}", block }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CourseDetailScreen(group_id: String, focus: Option<String>) -> Element {
    let app = use_app();
    let navigator = use_navigator();
    let api = app.api.clone();
    let resource_api = api.clone();
    let data_version = app.data_version;
    let current_group = group_id.clone();
    let course = use_resource(use_reactive((&current_group,), move |(group_id,)| {
        let _ = data_version();
        let api = resource_api.clone();
        async move { api.get_course(&group_id).await }
    }));

    let state = course.cloned();
    let title = match &state {
        Some(Ok(detail)) => detail.group.name.clone(),
        _ => t("courseDetail.fallback"),
    };
    let subtitle = match &state {
        Some(Ok(detail)) => Some(detail.group.code.clone()),
        _ => None,
    };

    let actions = rsx! {
        button {
            r#type: "button",
            class: "text-sm text-primary underline-offset-4 hover:underline",
            onclick: move |_| navigator.go_back(),
            "{t(\"courseDetail.back\")}"
        }
    };

    let on_open = {
        let api = api.clone();
        let toasts = app.toasts.clone();
        move |url: String| {
            let api = api.clone();
            let toasts = toasts.clone();
            spawn_future(async move {
                open_in_app(&api, &toasts, &url).await;
            });
        }
    };

    let body = match state {
        Some(Ok(detail)) => rsx! {
            CourseDetailView { detail, focus, on_open }
        },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title, subtitle, actions,
            {body}
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn scroll_to_focus(focus: Option<&str>) {
    let Some(focus) = focus else {
        return;
    };
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(element) = document.get_element_by_id(&format!("focus-{focus}")) else {
        return;
    };

    let options = web_sys::ScrollIntoViewOptions::new();
    options.set_block(web_sys::ScrollLogicalPosition::Center);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

#[cfg(not(target_arch = "wasm32"))]
fn scroll_to_focus(_focus: Option<&str>) {}
