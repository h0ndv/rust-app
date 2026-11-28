use contracts::{NewsItemDto, NewsKind};
use dioxus::prelude::*;

use crate::api::Api;
use crate::components::{EmptyNote, ErrorNote, Loading, Screen};
use crate::domain::{ITEM_KIND_KEY, ItemKind, item_kind};
use crate::i18n::t;
use crate::router::Route;
use crate::state::{spawn_future, use_app};

/// Course detail for the item. Announcements open the course without a focus;
/// every other kind scrolls to its own row.
pub fn news_route(item: &NewsItemDto) -> Route {
    let Some(group_id) = item.group_id.clone() else {
        return Route::Courses {};
    };

    let focus =
        (item_kind(Some(&item.kind)) != Some(ItemKind::Announcement)).then(|| item.id.clone());

    Route::CourseDetail { group_id, focus }
}

pub fn news_link(item: &NewsItemDto) -> String {
    let link = news_route(item).to_string();

    // The optional query leaves a bare `?` when it is absent; the React href
    // does not carry it.
    link.strip_suffix('?').unwrap_or(&link).to_string()
}

/// The command only accepts the three backend kinds; resource and unknown
/// values stay on the closest valid one.
fn news_kind(item: &NewsItemDto) -> NewsKind {
    match item_kind(Some(&item.kind)) {
        Some(ItemKind::Announcement) => NewsKind::Announcement,
        Some(ItemKind::File | ItemKind::Resource) => NewsKind::File,
        _ => NewsKind::Activity,
    }
}

pub async fn mark_read(api: &Api, item: &NewsItemDto) -> bool {
    api.acknowledge(news_kind(item), &item.id).await.is_ok()
}

#[component]
pub fn NewsItemBody(
    item: NewsItemDto,
    #[props(default)] link: Option<Route>,
    #[props(default)] busy: bool,
    on_acknowledge: EventHandler<NewsItemDto>,
) -> Element {
    let kind = item_kind(Some(&item.kind)).unwrap_or(ItemKind::Other);
    let label = t(ITEM_KIND_KEY[kind as usize]);
    let title = item.title.clone();
    let course = item.course.clone();
    let body = item.body.clone();
    let acknowledge = item.clone();

    rsx! {
        li { class: "flex flex-col gap-1 p-4",
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
            if let Some(course) = course.filter(|course| !course.is_empty()) {
                span { class: "text-xs text-muted-foreground", "{course}" }
            }
            if !body.is_empty() {
                p { class: "text-sm text-muted-foreground", "{body}" }
            }
            div { class: "flex justify-end",
                button {
                    r#type: "button",
                    class: "text-xs text-primary underline-offset-4 hover:underline disabled:opacity-50",
                    disabled: busy,
                    onclick: move |_| on_acknowledge.call(acknowledge.clone()),
                    "{t(\"common.markRead\")}"
                }
            }
        }
    }
}

#[component]
pub fn NewsView(
    items: Vec<NewsItemDto>,
    #[props(default)] busy: Option<String>,
    on_acknowledge: EventHandler<NewsItemDto>,
) -> Element {
    if items.is_empty() {
        return rsx! {
            EmptyNote { "{t(\"news.empty\")}" }
        };
    }

    rsx! {
        ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
            for item in items {
                NewsItemBody {
                    key: "{item.id}",
                    busy: busy.as_deref() == Some(item.id.as_str()),
                    link: item.group_id.is_some().then(|| news_route(&item)),
                    item,
                    on_acknowledge,
                }
            }
        }
    }
}

#[component]
pub fn NewsScreen() -> Element {
    let app = use_app();
    let resource_api = app.api.clone();
    let data_version = app.data_version;
    let mut busy = use_signal(|| None::<String>);

    let news = use_resource(move || {
        let _ = data_version();
        let api = resource_api.clone();
        async move { api.list_news().await }
    });

    let on_acknowledge = {
        let api = app.api.clone();
        let mut data_version = app.data_version;
        move |item: NewsItemDto| {
            busy.set(Some(item.id.clone()));
            let api = api.clone();

            spawn_future(async move {
                if mark_read(&api, &item).await {
                    let next = (*data_version.peek()).wrapping_add(1);
                    data_version.set(next);
                }
                busy.set(None);
            });
        }
    };

    let body = match news.cloned() {
        Some(Ok(items)) => rsx! {
            NewsView { items, busy: busy(), on_acknowledge }
        },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("news.title"), subtitle: t("news.subtitle"),
            {body}
        }
    }
}
