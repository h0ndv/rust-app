use std::rc::Rc;

use contracts::{ResourceDto, ResourceFilter};
use dioxus::prelude::*;

use crate::api::Api;
use crate::components::{EmptyNote, ErrorNote, Loading, Screen, Toast, ToastKind, ToastSink};
use crate::domain::{ITEM_KIND_KEY, ItemKind, item_kind};
use crate::events::{on_download_error, on_download_finished};
use crate::format::format_size;
use crate::i18n::{Language, resolve_language, t};
use crate::open::open_in_app;
use crate::state::{spawn_future, use_app};

fn active_language() -> Language {
    resolve_language(&rust_i18n::locale())
}

fn kind_key(kind: &str) -> &'static str {
    let kind = item_kind(Some(kind)).unwrap_or(ItemKind::Other);
    ITEM_KIND_KEY[kind as usize]
}

#[component]
fn ResourceRow(
    resource: ResourceDto,
    language: Language,
    #[props(default)] busy: bool,
    on_open: EventHandler<String>,
    on_download: EventHandler<String>,
) -> Element {
    let ResourceDto {
        id,
        title,
        kind,
        size,
        url,
        is_new,
        downloaded,
        ..
    } = resource;
    let kind = kind_key(&kind);
    let size = format_size(size, language);

    rsx! {
        li { class: "flex flex-wrap items-center gap-3 p-3 text-sm",
            span { class: "min-w-0 flex-1 truncate", "{title}" }
            if is_new {
                span { class: "rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary",
                    "{t(\"common.new\")}"
                }
            }
            span { class: "text-xs text-muted-foreground uppercase", "{t(kind)}" }
            span { class: "text-xs text-muted-foreground", "{size}" }
            if downloaded {
                span { class: "text-xs text-emerald-600", "{t(\"resources.downloaded\")}" }
            } else {
                button {
                    r#type: "button",
                    class: "text-xs text-primary underline-offset-4 hover:underline disabled:opacity-50",
                    disabled: busy,
                    onclick: move |_| on_download.call(id.clone()),
                    if busy {
                        "{t(\"resources.downloading\")}"
                    } else {
                        "{t(\"resources.download\")}"
                    }
                }
            }
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
pub fn ResourcesView(
    resources: Vec<ResourceDto>,
    #[props(default)] busy: Option<String>,
    on_open: EventHandler<String>,
    on_download: EventHandler<String>,
) -> Element {
    let language = active_language();

    if resources.is_empty() {
        return rsx! {
            EmptyNote { "{t(\"resources.empty\")}" }
        };
    }

    let no_course = t("resources.noCourse");
    let mut groups: Vec<(String, Vec<ResourceDto>)> = Vec::new();
    for resource in resources {
        let course = resource.course.clone().unwrap_or_else(|| no_course.clone());
        match groups.iter_mut().find(|(name, _)| *name == course) {
            Some((_, items)) => items.push(resource),
            None => groups.push((course, vec![resource])),
        }
    }

    rsx! {
        div { class: "flex flex-col gap-8",
            for (course, items) in groups {
                section { key: "{course}", class: "flex flex-col gap-2",
                    h3 { class: "text-sm font-semibold", "{course}" }
                    ul { class: "flex flex-col divide-y divide-border rounded-lg border border-border",
                        for resource in items {
                            ResourceRow {
                                key: "{resource.id}",
                                busy: busy.as_deref() == Some(resource.id.as_str()),
                                resource,
                                language,
                                on_open,
                                on_download,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Queues a download and turns a rejection into the localized toast. Returns
/// whether the enqueue was accepted; the worker reports completion through
/// `download://finished` and `download://error`.
pub async fn download_file(api: &Api, toasts: &dyn ToastSink, file_id: &str) -> bool {
    match api.download_file(file_id).await {
        Ok(()) => true,
        Err(error) => {
            toasts.push(Toast::new(ToastKind::Error, t(error.key())));
            false
        }
    }
}

#[component]
pub fn ResourcesScreen() -> Element {
    let app = use_app();
    let resource_api = app.api.clone();
    let data_version = app.data_version;
    let mut busy = use_signal(|| None::<String>);

    let resources = use_resource(move || {
        let _ = data_version();
        let api = resource_api.clone();
        async move { api.list_resources(ResourceFilter::All).await }
    });

    let mut finished_busy = busy;
    let mut finished_version = app.data_version;
    let toasts = app.toasts.clone();
    use_hook(move || {
        let finished = on_download_finished(move |file_id: String| {
            if finished_busy.peek().as_deref() == Some(file_id.as_str()) {
                finished_busy.set(None);
            }

            let next = (*finished_version.peek()).wrapping_add(1);
            finished_version.set(next);
        });

        let mut error_busy = busy;
        let error_toasts = toasts;
        let failed = on_download_error(move |file_id: String| {
            if error_busy.peek().as_deref() == Some(file_id.as_str()) {
                error_busy.set(None);
            }

            error_toasts.push(Toast::new(ToastKind::Error, t("resources.downloadFailed")));
        });

        Rc::new((finished, failed))
    });

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

    let on_download = {
        let api = app.api.clone();
        let toasts = app.toasts.clone();
        move |id: String| {
            if busy.peek().as_deref() == Some(id.as_str()) {
                return;
            }

            busy.set(Some(id.clone()));
            let api = api.clone();
            let toasts = toasts.clone();
            let mut busy = busy;
            spawn_future(async move {
                if !download_file(&api, &toasts, &id).await {
                    busy.set(None);
                }
            });
        }
    };

    let body = match resources.cloned() {
        Some(Ok(resources)) => rsx! {
            ResourcesView { resources, busy: busy(), on_open, on_download }
        },
        Some(Err(error)) => rsx! { ErrorNote { kind: error.kind } },
        None => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("resources.title"), subtitle: t("resources.subtitle"),
            {body}
        }
    }
}
