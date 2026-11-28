use dioxus::prelude::*;

use crate::error::{ERROR_KEY, ErrorKind};
use crate::i18n::t;

#[component]
pub fn Screen(
    title: String,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] actions: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        section { class: "flex flex-col gap-6",
            header { class: "flex flex-wrap items-start justify-between gap-3",
                div { class: "flex flex-col gap-1",
                    h2 { class: "text-xl font-semibold tracking-tight", "{title}" }
                    if let Some(subtitle) = subtitle {
                        p { class: "text-sm text-muted-foreground", "{subtitle}" }
                    }
                }
                if let Some(actions) = actions {
                    {actions}
                }
            }
            {children}
        }
    }
}

#[component]
pub fn Loading() -> Element {
    rsx! {
        p { class: "text-sm text-muted-foreground", "{t(\"common.loading\")}" }
    }
}

#[component]
pub fn ErrorNote(kind: ErrorKind) -> Element {
    let message = t(ERROR_KEY[kind as usize]);

    rsx! {
        p {
            role: "alert",
            class: "rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive",
            "{message}"
        }
    }
}

#[component]
pub fn EmptyNote(children: Element) -> Element {
    rsx! {
        p { class: "rounded-lg border border-dashed border-border p-6 text-sm text-muted-foreground",
            {children}
        }
    }
}
