use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::prelude::*;

use crate::i18n::t;
use crate::icons;

#[cfg(target_arch = "wasm32")]
const LIFETIME_MS: u32 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub title: String,
    pub description: Option<String>,
}

impl Toast {
    pub fn new(kind: ToastKind, title: impl Into<String>) -> Self {
        Self {
            id: 0,
            kind,
            title: title.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

pub trait ToastSink {
    fn push(&self, toast: Toast);
}

#[derive(Clone)]
pub struct ToastStore {
    items: Rc<RefCell<Vec<Toast>>>,
    next_id: Rc<Cell<u64>>,
    revision: Signal<u64>,
}

impl PartialEq for ToastStore {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.items, &other.items)
    }
}

impl ToastStore {
    /// Must run inside a Dioxus scope: it creates the revision signal, so there
    /// is deliberately no `Default` implementation.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            items: Rc::new(RefCell::new(Vec::new())),
            next_id: Rc::new(Cell::new(0)),
            revision: Signal::new(0),
        }
    }

    /// Reads the revision so the calling scope re-renders whenever the queue
    /// changes.
    pub fn items(&self) -> Vec<Toast> {
        let _ = self.revision.read();
        self.items.borrow().clone()
    }

    pub fn dismiss(&self, id: u64) {
        let mut items = self.items.borrow_mut();
        let before = items.len();
        items.retain(|toast| toast.id != id);
        let changed = items.len() != before;
        drop(items);

        if changed {
            self.bump();
        }
    }

    fn next_id(&self) -> u64 {
        let id = self.next_id.get().wrapping_add(1);
        self.next_id.set(id);
        id
    }

    fn bump(&self) {
        let next = (*self.revision.peek()).wrapping_add(1);
        let mut revision = self.revision;
        revision.set(next);
    }
}

impl ToastSink for ToastStore {
    fn push(&self, mut toast: Toast) {
        toast.id = self.next_id();
        self.items.borrow_mut().push(toast);
        self.bump();
    }
}

#[component]
pub fn ToastViewport(store: ToastStore) -> Element {
    let toasts = store.items();

    rsx! {
        div {
            class: "pointer-events-none fixed bottom-4 right-4 z-50 flex w-80 max-w-[calc(100%-2rem)] flex-col gap-2",
            aria_live: "polite",
            for toast in toasts {
                ToastCard { key: "{toast.id}", toast, store: store.clone() }
            }
        }
    }
}

#[component]
fn ToastCard(toast: Toast, store: ToastStore) -> Element {
    let id = toast.id;
    let timeout_store = store.clone();
    let dismiss_store = store;

    use_future(move || {
        let store = timeout_store.clone();
        async move {
            #[cfg(target_arch = "wasm32")]
            {
                gloo_timers::future::TimeoutFuture::new(LIFETIME_MS).await;
                store.dismiss(id);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (&store, id);
            }
        }
    });

    let icon = match toast.kind {
        ToastKind::Info => icons::circle("size-4 text-info"),
        ToastKind::Success => icons::check("size-4 text-success"),
        ToastKind::Warning => icons::circle("size-4 text-warning"),
        ToastKind::Error => icons::x("size-4 text-destructive"),
    };

    rsx! {
        div {
            role: "status",
            class: "pointer-events-auto flex w-full items-start gap-3 rounded-lg border border-border bg-popover p-3 text-sm text-popover-foreground shadow-lg",
            span { class: "mt-0.5", {icon} }
            div { class: "flex min-w-0 flex-1 flex-col gap-0.5",
                span { class: "font-medium", "{toast.title}" }
                if let Some(description) = toast.description.clone() {
                    span { class: "text-xs text-muted-foreground", "{description}" }
                }
            }
            button {
                r#type: "button",
                class: "rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted",
                aria_label: t("recovery.dismiss"),
                onclick: move |_| dismiss_store.dismiss(id),
                {icons::x("size-3")}
            }
        }
    }
}
