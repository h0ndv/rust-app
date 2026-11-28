use contracts::{SearchHitDto, SearchResultsDto};
use dioxus::prelude::*;

use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::domain::{ITEM_KIND_KEY, ItemKind, item_kind};
use crate::i18n::t;
use crate::router::Route;
use crate::state::use_app;

pub fn normalize_query(input: &str) -> String {
    input.trim().to_string()
}

pub fn search_route(hit: &SearchHitDto) -> Route {
    let Some(group_id) = hit.group_id.clone() else {
        return Route::Courses {};
    };

    let focus =
        (item_kind(Some(&hit.kind)) != Some(ItemKind::Announcement)).then(|| hit.id.clone());

    Route::CourseDetail { group_id, focus }
}

pub fn search_link(hit: &SearchHitDto) -> String {
    let link = search_route(hit).to_string();

    // The optional query leaves a bare `?` when it is absent; the React href
    // does not carry it.
    link.strip_suffix('?').unwrap_or(&link).to_string()
}

fn no_results(query: &str) -> String {
    t("search.noResults").replace("%{query}", query)
}

#[component]
fn SearchHit(hit: SearchHitDto) -> Element {
    let kind = item_kind(Some(&hit.kind)).unwrap_or(ItemKind::Other);
    let label = t(ITEM_KIND_KEY[kind as usize]);
    let title = hit.title.clone();
    let body = hit.body.clone();
    let link = hit.group_id.is_some().then(|| search_route(&hit));

    rsx! {
        li { class: "flex flex-col gap-1 p-3",
            div { class: "flex flex-wrap items-center gap-2",
                span { class: "rounded bg-muted px-2 py-0.5 text-xs font-medium uppercase",
                    "{label}"
                }
                span { class: "text-sm font-medium",
                    if let Some(route) = link {
                        Link {
                            to: route,
                            title: t("common.openCourse"),
                            class: "underline-offset-4 hover:underline",
                            "{title}"
                        }
                    } else {
                        "{title}"
                    }
                }
            }
            if !body.is_empty() {
                p { class: "text-xs text-muted-foreground", "{body}" }
            }
        }
    }
}

#[component]
pub fn SearchResultsView(results: SearchResultsDto) -> Element {
    if results.total == 0 {
        return rsx! {
            EmptyNote { "{no_results(&results.query)}" }
        };
    }

    rsx! {
        ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
            for hit in results.items {
                SearchHit { key: "{hit.kind}-{hit.id}", hit }
            }
        }
    }
}

#[component]
pub fn SearchScreen() -> Element {
    let app = use_app();
    let resource_api = app.api.clone();
    let data_version = app.data_version;
    let mut input = use_signal(String::new);
    let mut query = use_signal(String::new);

    let results = use_resource(move || {
        let _ = data_version();
        let query = query();
        let api = resource_api.clone();
        async move {
            if query.is_empty() {
                return None;
            }

            Some(api.search(&query).await)
        }
    });

    let pending = matches!(*results.state().read(), UseResourceState::Pending);

    let body = if query().is_empty() {
        rsx! {
            EmptyNote { "{t(\"search.emptyQuery\")}" }
        }
    } else if pending {
        rsx! { Loading {} }
    } else {
        match results.cloned() {
            Some(Some(Ok(results))) => rsx! { SearchResultsView { results } },
            Some(Some(Err(error))) => rsx! { ErrorNote { kind: error.kind } },
            _ => rsx! { Loading {} },
        }
    };

    rsx! {
        Screen { title: t("search.title"), subtitle: t("search.subtitle"),
            form {
                class: "flex gap-2",
                onsubmit: move |event| {
                    event.prevent_default();
                    query.set(normalize_query(&input()));
                },
                input {
                    class: "flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm",
                    placeholder: t("search.placeholder"),
                    value: "{input}",
                    oninput: move |event| input.set(event.value()),
                }
                button {
                    r#type: "submit",
                    class: "rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground",
                    "{t(\"search.button\")}"
                }
            }
            {body}
        }
    }
}
