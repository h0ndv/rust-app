use contracts::{CourseGroupDto, CourseSectionDto};
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::format::format_date_local;
use crate::i18n::{Language, resolve_language, t, tn};
use crate::router::Route;
use crate::state::use_app;

const GRID: &str = "grid gap-4 md:grid-cols-2 xl:grid-cols-3";
const CARD: &str = "flex flex-col gap-3 rounded-xl border border-border p-4";
const LINK: &str = "text-sm text-primary underline-offset-4 hover:underline";

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

pub fn course_route(group_id: &str) -> Route {
    Route::CourseDetail {
        group_id: group_id.into(),
        focus: None,
    }
}

#[component]
fn SectionLine(section: CourseSectionDto) -> Element {
    let language = active_language();
    let label = section.section.unwrap_or_else(|| t("courses.section"));
    let teacher = section.teacher.unwrap_or(section.fullname);
    let pending = (section.pending_tasks > 0).then(|| tn("courses.pending", section.pending_tasks));
    let due = section
        .next_due_at
        .map(|due_at| format_date_local(Some(due_at), language));

    rsx! {
        li { class: "flex flex-wrap items-center gap-2 text-sm",
            span { class: "rounded bg-muted px-2 py-0.5 text-xs font-medium", "{label}" }
            span { class: "min-w-0 flex-1 truncate text-muted-foreground", "{teacher}" }
            if let Some(pending) = pending {
                span { class: "text-xs text-muted-foreground", "{pending}" }
            }
            if let Some(due) = due {
                span { class: "text-xs text-muted-foreground", "{due}" }
            }
        }
    }
}

#[component]
fn CourseCardBody(group: CourseGroupDto) -> Element {
    let CourseGroupDto {
        code,
        name,
        sections,
        ..
    } = group;

    rsx! {
        header { class: "flex flex-col gap-1",
            span { class: "text-xs font-medium tracking-wide text-muted-foreground uppercase",
                "{code}"
            }
            h3 { class: "text-base font-semibold", "{name}" }
        }
        ul { class: "flex flex-col gap-2",
            for section in sections {
                SectionLine { key: "{section.id}", section }
            }
        }
    }
}

#[component]
fn CourseCard(group: CourseGroupDto) -> Element {
    rsx! {
        article { class: CARD,
            CourseCardBody { group }
        }
    }
}

#[component]
fn CourseCardLink(group: CourseGroupDto) -> Element {
    let route = course_route(&group.id);

    rsx! {
        article { class: CARD,
            CourseCardBody { group }
            Link { to: route, class: LINK, "{t(\"courses.viewDetail\")}" }
        }
    }
}

#[component]
pub fn CoursesGrid(groups: Vec<CourseGroupDto>) -> Element {
    if groups.is_empty() {
        return rsx! {
            EmptyNote { "{t(\"courses.empty\")}" }
        };
    }

    rsx! {
        div { class: GRID,
            for group in groups {
                CourseCard { key: "{group.id}", group }
            }
        }
    }
}

#[component]
pub fn CoursesView(groups: Vec<CourseGroupDto>) -> Element {
    if groups.is_empty() {
        return rsx! {
            EmptyNote { "{t(\"courses.empty\")}" }
        };
    }

    rsx! {
        div { class: GRID,
            for group in groups {
                CourseCardLink { key: "{group.id}", group }
            }
        }
    }
}

#[component]
pub fn CoursesScreen() -> Element {
    let app = use_app();
    let api = app.api.clone();
    let data_version = app.data_version;
    let courses = use_resource(move || {
        let _ = data_version();
        let api = api.clone();
        async move { api.list_courses().await }
    });

    let body = match courses.cloned() {
        Some(Ok(groups)) => rsx! { CoursesView { groups } },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("courses.title"), subtitle: t("courses.subtitle"),
            {body}
        }
    }
}
