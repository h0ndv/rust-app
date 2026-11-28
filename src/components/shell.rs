use contracts::{RecoveryNoticeDto, SessionState, Source};
use dioxus::prelude::*;

use crate::components::primitives::{Badge, BadgeVariant, Button, ButtonVariant, Progress};
use crate::components::sidebar::Sidebar;
use crate::domain::{SyncState, source_label};
use crate::i18n::t;
use crate::router::Route;
use crate::state::{
    SessionHandle, SyncController, use_due_notifications, use_session, use_sync_controller,
};

#[component]
pub fn AppShell() -> Element {
    let current = use_route::<Route>();
    let mut collapsed = use_signal(|| false);
    let is_collapsed = collapsed();
    let SyncController {
        view,
        start,
        cancel,
    } = use_sync_controller();
    let SessionHandle {
        state: session,
        connecting,
        connect,
    } = use_session();
    use_due_notifications();

    let running = view.state == SyncState::Running;
    let source = view.source.map(source_label);
    let last_update = view
        .last_run
        .as_ref()
        .and_then(|run| run.finished_at.as_deref())
        .and_then(clock_time)
        .map(|time| t("shell.lastUpdate").replace("%{time}", &time));
    let sync_label = if running {
        t("shell.syncing")
    } else {
        t("shell.sync")
    };

    rsx! {
        div { class: "min-h-svh bg-background text-foreground",
            Sidebar {
                collapsed: is_collapsed,
                current,
                on_toggle: move |_| collapsed.toggle(),
            }
            div {
                class: if is_collapsed {
                    "ml-14 flex min-h-svh flex-col transition-[margin] duration-200"
                } else {
                    "ml-64 flex min-h-svh flex-col transition-[margin] duration-200"
                },
                header { class: "flex h-14 items-center gap-3 border-b border-border px-4",
                    SyncBadge { state: view.state, progress: view.progress }
                    if running {
                        div { class: "flex items-center gap-2",
                            Progress { value: view.progress, class: "w-24" }
                            if let Some(source) = source {
                                span { class: "text-xs text-muted-foreground", "{source}" }
                            }
                        }
                    }
                    if let Some(text) = last_update {
                        span { class: "text-xs text-muted-foreground", "{text}" }
                    }
                    div { class: "ml-auto flex items-center gap-2",
                        if running {
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| cancel.call(()),
                                "{t(\"common.cancel\")}"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: running,
                            onclick: move |_| start.call(()),
                            "{sync_label}"
                        }
                    }
                }
                main { class: "flex-1 overflow-auto p-6",
                    SessionBanner {
                        state: session,
                        connecting,
                        on_connect: move |source| connect.call(source),
                    }
                    Outlet::<Route> {}
                }
            }
        }
    }
}

fn clock_time(finished_at: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(finished_at)
        .ok()
        .map(|when| {
            when.with_timezone(&chrono::Local)
                .format("%H:%M")
                .to_string()
        })
}

#[component]
pub fn SyncBadge(state: SyncState, progress: u8) -> Element {
    let label = match state {
        SyncState::Idle => t("shell.notSynced"),
        SyncState::Running => t("shell.syncStatus.running"),
        SyncState::Ok => t("shell.syncStatus.ok"),
        SyncState::Partial => t("shell.syncStatus.partial"),
        SyncState::Failed => t("shell.syncStatus.failed"),
    };
    let variant = if state == SyncState::Running {
        BadgeVariant::Secondary
    } else {
        BadgeVariant::Outline
    };

    rsx! {
        Badge { variant,
            "{label}"
            if state == SyncState::Running {
                span { class: "text-muted-foreground", "{progress}%" }
            }
        }
    }
}

#[component]
pub fn SessionBanner(
    state: Option<SessionState>,
    connecting: bool,
    on_connect: EventHandler<Option<Source>>,
) -> Element {
    let Some(state) = state else {
        return rsx! {};
    };

    if connecting || matches!(&state, SessionState::Connecting(_)) {
        return rsx! {
            Banner {
                title: t("session.connectingTitle"),
                body: t("session.connectingBody"),
            }
        };
    }

    match state {
        SessionState::Connected => rsx! {},
        SessionState::Disconnected => rsx! {
            Banner {
                title: t("session.disconnectedTitle"),
                body: t("session.disconnectedBody"),
                action: t("session.connect"),
                on_action: move |_| on_connect.call(None),
            }
        },
        SessionState::Expired(source) => rsx! {
            Banner {
                title: t("session.expiredTitle"),
                body: t("session.expiredBody").replace("%{source}", source_label(source)),
                action: t("session.reconnect"),
                on_action: move |_| on_connect.call(Some(source)),
            }
        },
        SessionState::Partial => rsx! {
            Banner {
                title: t("session.partialTitle"),
                body: t("session.partialBody"),
                action: t("session.reconnect"),
                on_action: move |_| on_connect.call(None),
            }
        },
        SessionState::Connecting(_) => rsx! {},
    }
}

#[component]
pub fn RecoveryBanner(notice: RecoveryNoticeDto, on_dismiss: EventHandler<()>) -> Element {
    let title = t("recovery.title");
    let body = t("recovery.body").replace("%{path}", &notice.backup_path);
    let dismiss = t("recovery.dismiss");

    rsx! {
        div {
            role: "status",
            class: "m-4 flex flex-wrap items-center gap-3 rounded-lg border border-destructive/40 bg-destructive/5 p-3 text-sm",
            div { class: "flex min-w-0 flex-1 flex-col gap-1",
                span { class: "font-medium", "{title}" }
                span { class: "text-xs text-muted-foreground", "{body}" }
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| on_dismiss.call(()),
                "{dismiss}"
            }
        }
    }
}

#[component]
fn Banner(
    title: String,
    body: String,
    #[props(default)] action: Option<String>,
    #[props(default)] on_action: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            role: "status",
            class: "mb-6 flex flex-wrap items-center gap-3 rounded-lg border border-primary/30 bg-primary/5 p-3 text-sm",
            div { class: "flex min-w-0 flex-1 flex-col gap-1",
                span { class: "font-medium", "{title}" }
                span { class: "text-xs text-muted-foreground", "{body}" }
            }
            if let Some(action) = action {
                Button {
                    onclick: move |event| on_action.call(event),
                    "{action}"
                }
            }
        }
    }
}
