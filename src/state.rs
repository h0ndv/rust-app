use std::future::Future;
use std::rc::Rc;

use contracts::{SessionState, Source, SyncRun, SyncStatus, SyncStatusKind};
use dioxus::prelude::*;

use crate::api::Api;
use crate::components::ToastStore;
use crate::domain::SyncState;
use crate::events::{on_session_changed, on_sync_finished, on_sync_progress};
use crate::i18n::Language;
use crate::notifications::{ensure_notification_permission, should_notify};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncView {
    pub state: SyncState,
    pub progress: u8,
    pub source: Option<Source>,
    pub requests: u32,
    pub errors: Vec<String>,
    pub last_run: Option<SyncRun>,
}

#[derive(Clone)]
pub struct AppContext {
    pub api: Api,
    pub sync: Signal<SyncView>,
    pub session: Signal<Option<SessionState>>,
    pub data_version: Signal<u64>,
    pub toasts: ToastStore,
    pub language: Signal<Language>,
}

impl AppContext {
    pub fn set_language(&self, language: Language) {
        crate::i18n::set_language(language);

        let mut signal = self.language;
        signal.set(language);
    }
}

pub fn use_app() -> AppContext {
    use_context::<AppContext>()
}

pub fn apply_progress(view: &mut SyncView, status: SyncStatus) {
    view.state = sync_state(status.state);
    view.progress = status.progress;
    view.source = status.source;
    view.requests = status.requests;
    view.errors = status.errors;
}

pub fn apply_finished(view: &mut SyncView, run: SyncRun) {
    view.state = sync_state(run.status);
    view.progress = 100;
    view.requests = run.requests;
    view.last_run = Some(run);
}

fn sync_state(status: SyncStatusKind) -> SyncState {
    match status {
        SyncStatusKind::Running => SyncState::Running,
        SyncStatusKind::Ok => SyncState::Ok,
        SyncStatusKind::Partial => SyncState::Partial,
        SyncStatusKind::Failed => SyncState::Failed,
    }
}

#[derive(Clone)]
pub struct SyncController {
    pub view: SyncView,
    pub start: Callback<()>,
    pub cancel: Callback<()>,
}

pub fn use_sync_controller() -> SyncController {
    let app = use_app();

    let mut events = app.clone();
    use_hook(move || {
        let mut view = events.sync;
        let progress = on_sync_progress(move |status: SyncStatus| {
            let mut current = view.write();
            apply_progress(&mut current, status);
        });

        let mut view = events.sync;
        let mut data_version = events.data_version;
        let finished = on_sync_finished(move |run: SyncRun| {
            {
                let mut current = view.write();
                apply_finished(&mut current, run);
            }

            let next = (*data_version.peek()).wrapping_add(1);
            data_version.set(next);
        });

        let api = events.api.clone();
        spawn_future(async move {
            let Ok(status) = api.sync_status().await else {
                return;
            };
            if status.state == SyncStatusKind::Ok && status.progress == 0 {
                return;
            }

            let mut current = events.sync.write();
            apply_progress(&mut current, status);
        });

        Rc::new((progress, finished))
    });

    let start = {
        let api = app.api.clone();
        let mut view = app.sync;
        use_callback(move |_: ()| {
            let api = api.clone();
            spawn_future(async move {
                match api.start_sync().await {
                    Ok(status) => {
                        let mut current = view.write();
                        apply_progress(&mut current, status);
                    }
                    Err(_) => view.write().state = SyncState::Failed,
                }
            });
        })
    };

    let cancel = {
        let api = app.api.clone();
        use_callback(move |_: ()| {
            let api = api.clone();
            spawn_future(async move {
                let _ = api.cancel_sync().await;
            });
        })
    };

    SyncController {
        view: (*app.sync.read()).clone(),
        start,
        cancel,
    }
}

#[derive(Clone)]
pub struct SessionHandle {
    pub state: Option<SessionState>,
    pub connecting: bool,
    pub connect: Callback<Option<Source>>,
}

pub fn use_session() -> SessionHandle {
    let app = use_app();
    let connecting = use_signal(|| false);

    let mut events = app.clone();
    use_hook(move || {
        let mut state = events.session;
        let changed = on_session_changed(move |next: SessionState| state.set(Some(next)));

        let api = events.api.clone();
        spawn_future(async move {
            let next = api
                .session_state()
                .await
                .unwrap_or(SessionState::Disconnected);
            if events.session.peek().is_none() {
                events.session.set(Some(next));
            }
        });

        Rc::new(changed)
    });

    let connect = {
        let api = app.api.clone();
        let mut state = app.session;
        let mut busy = connecting;
        use_callback(move |source: Option<Source>| {
            let api = api.clone();
            busy.set(true);
            spawn_future(async move {
                if let Ok(next) = api.connect(source).await {
                    state.set(Some(next));
                }
                busy.set(false);
            });
        })
    };

    SessionHandle {
        state: (*app.session.read()).clone(),
        connecting: connecting(),
        connect,
    }
}

/// After a sync the shell may announce overdue or soon-due work. Preferences
/// and the dashboard are read fresh so a settings change lands immediately.
pub fn use_due_notifications() {
    let app = use_app();

    use_hook(move || {
        let api = app.api.clone();
        let listener = on_sync_finished(move |run: SyncRun| {
            if run.status == SyncStatusKind::Failed {
                return;
            }

            let api = api.clone();
            spawn_future(async move {
                let Ok(settings) = api.get_settings().await else {
                    return;
                };
                if !settings.notifications {
                    return;
                }

                let Ok(dashboard) = api.get_dashboard().await else {
                    return;
                };
                let Some((title, body)) = should_notify(&settings, &dashboard) else {
                    return;
                };

                if ensure_notification_permission(&api).await {
                    let _ = api.send_notification(&title, &body).await;
                }
            });
        });

        Rc::new(listener)
    });
}

/// Runs an async side effect from render or event code. Native builds only run
/// tests, which never have a live transport, so the future is dropped.
pub(crate) fn spawn_future(future: impl Future<Output = ()> + 'static) {
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::mem::drop(future);
    }
}
