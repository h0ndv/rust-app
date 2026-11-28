/// Bookkeeping shared by the JS listener and native tests. The closure is only
/// released after its unlisten call ran, so a pending `listen` cannot leave
/// Tauri holding a freed callback.
#[cfg(any(target_arch = "wasm32", test))]
struct Shared<C, U> {
    closure: Option<C>,
    unlisten: Option<U>,
    dropped: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, PartialEq, Eq)]
enum ResolveOutcome<U> {
    Stored,
    CallNow(U),
}

#[cfg(any(target_arch = "wasm32", test))]
impl<C, U> Shared<C, U> {
    fn new() -> Self {
        Self {
            closure: None,
            unlisten: None,
            dropped: false,
        }
    }

    /// The guard is gone. A ready unlisten is returned so the caller can run it
    /// before releasing the closure; a pending one stays in the resolver.
    fn drop_guard(&mut self) -> Option<U> {
        self.dropped = true;
        self.unlisten.take()
    }

    /// Stores a resolved unlisten, or asks the caller to run it now because the
    /// guard disappeared first.
    fn resolve(&mut self, unlisten: U) -> ResolveOutcome<U> {
        if self.dropped {
            ResolveOutcome::CallNow(unlisten)
        } else {
            self.unlisten = Some(unlisten);
            ResolveOutcome::Stored
        }
    }

    fn release_closure(&mut self) {
        self.closure = None;
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use js_sys::wasm_bindgen::closure::Closure;
    use js_sys::wasm_bindgen::{JsCast, JsValue};
    use serde::de::DeserializeOwned;

    use super::{ResolveOutcome, Shared};

    type Callback = Closure<dyn FnMut(JsValue)>;
    type SharedState = Shared<Callback, js_sys::Function>;

    /// Keeps a listener alive. Dropping it unlistens, even if the async
    /// `listen` call has not resolved yet.
    pub struct Listener {
        shared: Rc<RefCell<SharedState>>,
    }

    impl Listener {
        pub fn new<T>(event_name: &str, mut handler: impl FnMut(T) + 'static) -> Self
        where
            T: DeserializeOwned + 'static,
        {
            let shared = Rc::new(RefCell::new(SharedState::new()));

            let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &JsValue::from_str("payload"))
                    .unwrap_or(JsValue::UNDEFINED);

                if let Ok(payload) = serde_wasm_bindgen::from_value::<T>(payload) {
                    handler(payload);
                }
            });
            let callback = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
            shared.borrow_mut().closure = Some(closure);

            if let Some(promise) = listen(event_name, &callback) {
                let state = Rc::clone(&shared);
                wasm_bindgen_futures::spawn_local(async move {
                    let Ok(unlisten) = wasm_bindgen_futures::JsFuture::from(promise).await else {
                        return;
                    };
                    let Ok(unlisten) = unlisten.dyn_into::<js_sys::Function>() else {
                        return;
                    };

                    let call_now = match state.borrow_mut().resolve(unlisten) {
                        ResolveOutcome::Stored => None,
                        ResolveOutcome::CallNow(unlisten) => Some(unlisten),
                    };

                    if let Some(unlisten) = call_now {
                        let _ = unlisten.call0(&JsValue::UNDEFINED);
                        state.borrow_mut().release_closure();
                    }
                });
            }

            Self { shared }
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            let unlisten = self.shared.borrow_mut().drop_guard();

            if let Some(unlisten) = unlisten {
                let _ = unlisten.call0(&JsValue::UNDEFINED);
                self.shared.borrow_mut().release_closure();
            }
        }
    }

    fn listen(event_name: &str, callback: &js_sys::Function) -> Option<js_sys::Promise> {
        let tauri =
            js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("__TAURI__")).ok()?;
        let events = js_sys::Reflect::get(&tauri, &JsValue::from_str("event")).ok()?;
        let listen = js_sys::Reflect::get(&events, &JsValue::from_str("listen")).ok()?;
        let listen: js_sys::Function = listen.dyn_into().ok()?;

        listen
            .call2(
                &JsValue::UNDEFINED,
                &JsValue::from_str(event_name),
                callback,
            )
            .ok()?
            .dyn_into()
            .ok()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    /// Native tests never receive events, so the guard owns nothing.
    pub struct Listener;

    impl Listener {
        pub fn new<T>(_event_name: &str, _handler: impl FnMut(T) + 'static) -> Self {
            Self
        }
    }
}

pub use imp::Listener;

use contracts::{SessionState, SyncRun, SyncStatus};

pub fn on_sync_progress(handler: impl FnMut(SyncStatus) + 'static) -> Listener {
    Listener::new("sync://progress", handler)
}

pub fn on_sync_finished(handler: impl FnMut(SyncRun) + 'static) -> Listener {
    Listener::new("sync://finished", handler)
}

pub fn on_session_changed(handler: impl FnMut(SessionState) + 'static) -> Listener {
    Listener::new("session://changed", handler)
}

/// Payload is the `file_id` of the download that landed.
pub fn on_download_finished(handler: impl FnMut(String) + 'static) -> Listener {
    Listener::new("download://finished", handler)
}

/// Payload is the `file_id` of the download that failed.
pub fn on_download_error(handler: impl FnMut(String) + 'static) -> Listener {
    Listener::new("download://error", handler)
}

#[cfg(test)]
mod tests {
    use super::{ResolveOutcome, Shared};

    fn listener<'a>(closure: &'a str, unlisten: Option<&'a str>) -> Shared<&'a str, &'a str> {
        let mut shared = Shared::new();
        shared.closure = Some(closure);
        shared.unlisten = unlisten;
        shared
    }

    /// The regression: dropping before `listen` resolves used to release the
    /// closure while Tauri still held the callback.
    #[test]
    fn dropping_before_listen_resolves_keeps_the_closure() {
        let mut listener = listener("callback", None);

        assert_eq!(listener.drop_guard(), None);
        assert!(listener.dropped);
        assert_eq!(listener.closure, Some("callback"));
    }

    #[test]
    fn a_late_resolve_unlistens_before_releasing_the_closure() {
        let mut listener = listener("callback", None);
        assert_eq!(listener.drop_guard(), None);

        assert_eq!(
            listener.resolve("unlisten"),
            ResolveOutcome::CallNow("unlisten")
        );
        assert_eq!(listener.closure, Some("callback"));

        listener.release_closure();
        assert_eq!(listener.closure, None);
    }

    #[test]
    fn a_resolve_before_the_drop_stores_the_unlisten() {
        let mut listener = listener("callback", None);

        assert_eq!(listener.resolve("unlisten"), ResolveOutcome::Stored);
        assert_eq!(listener.unlisten, Some("unlisten"));
        assert_eq!(listener.closure, Some("callback"));
    }

    #[test]
    fn dropping_after_the_resolve_returns_the_ready_unlisten() {
        let mut listener = listener("callback", None);
        assert_eq!(listener.resolve("unlisten"), ResolveOutcome::Stored);

        assert_eq!(listener.drop_guard(), Some("unlisten"));
        listener.release_closure();
        assert_eq!(listener.closure, None);
    }
}
