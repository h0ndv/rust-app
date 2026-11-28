use std::rc::Rc;

use contracts::{RecoveryNoticeDto, SessionState};
use dioxus::prelude::*;

use crate::api::Api;
use crate::components::{Loading, RecoveryBanner, ToastStore, ToastViewport};
use crate::features::onboarding::Onboarding;
use crate::i18n::{Language, resolve_language};
use crate::router::Route;
use crate::state::{AppContext, SyncView, spawn_future};
use crate::theme::init_theme;
#[cfg(target_arch = "wasm32")]
use crate::transport::TauriTransport;
use crate::transport::Transport;
#[cfg(not(target_arch = "wasm32"))]
use crate::transport::UnsupportedTransport;

static CSS: Asset = asset!("/assets/app.css");

// `app.css` points at `./fonts/*.woff2` relative URLs, so the folder has to be
// part of the bundle as well.
const _: Asset = asset!("/assets/fonts", AssetOptions::folder());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Loading,
    Onboarding,
    App,
}

pub fn startup_mode(has_data: bool, session: &SessionState) -> Mode {
    if has_data || *session != SessionState::Disconnected {
        Mode::App
    } else {
        Mode::Onboarding
    }
}

#[component]
pub fn App() -> Element {
    let api = use_hook(|| Api::new(default_transport()));
    let toasts = use_hook(ToastStore::new);
    let sync = use_signal(SyncView::default);
    let session = use_signal(|| None::<SessionState>);
    let data_version = use_signal(|| 0u64);
    let language = use_signal(|| Language::En);

    let context = AppContext {
        api: api.clone(),
        sync,
        session,
        data_version,
        toasts: toasts.clone(),
        language,
    };
    use_context_provider(|| context.clone());

    let mut mode = use_signal(|| Mode::Loading);
    let mut recovery = use_signal(|| None::<RecoveryNoticeDto>);
    let has_data = use_signal(|| None::<bool>);

    use_hook(move || start(context.clone(), mode, recovery, has_data));

    let active = mode();
    let code = language().code();

    rsx! {
        document::Stylesheet { href: CSS }
        ToastViewport { store: toasts }
        if let Some(notice) = recovery() {
            RecoveryBanner {
                notice,
                on_dismiss: move |_| recovery.set(None),
            }
        }
        match active {
            Mode::Loading => rsx! {
                main { class: "flex min-h-svh items-center justify-center bg-background p-6",
                    Loading {}
                }
            },
            Mode::Onboarding => rsx! {
                Onboarding { on_continue: move |_| mode.set(Mode::App) }
            },
            Mode::App => rsx! {
                Router::<Route> { key: "{code}" }
            },
        }
    }
}

fn start(
    app: AppContext,
    mut mode: Signal<Mode>,
    mut recovery: Signal<Option<RecoveryNoticeDto>>,
    mut has_data: Signal<Option<bool>>,
) {
    let settings_app = app.clone();
    spawn_future(async move {
        let settings = settings_app.api.get_settings().await.unwrap_or_default();
        settings_app.set_language(resolve_language(&settings.language));
        init_theme(&settings.theme, &settings_app.api);
    });

    let data = app.clone();
    spawn_future(async move {
        let value = data.api.has_local_data().await.unwrap_or(false);
        has_data.set(Some(value));

        let session = (*data.session.peek()).clone();
        if let Some(session) = session {
            mode.set(startup_mode(value, &session));
        }
    });

    let mut session = app.clone();
    spawn_future(async move {
        let state = session
            .api
            .session_state()
            .await
            .unwrap_or(SessionState::Disconnected);
        session.session.set(Some(state.clone()));

        if let Some(value) = *has_data.peek() {
            mode.set(startup_mode(value, &state));
        }
    });

    spawn_future(async move {
        let notice = app.api.recovery_notice().await.ok().flatten();
        recovery.set(notice);
    });
}

#[cfg(target_arch = "wasm32")]
fn default_transport() -> Rc<dyn Transport> {
    Rc::new(TauriTransport)
}

#[cfg(not(target_arch = "wasm32"))]
fn default_transport() -> Rc<dyn Transport> {
    Rc::new(UnsupportedTransport)
}
