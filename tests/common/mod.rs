#![allow(dead_code)] // Each suite compiles this module and only uses part of it.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

use async_trait::async_trait;
use rust_app_ui::components::{Toast, ToastSink};
use rust_app_ui::error::{UiError, app_error};
use rust_app_ui::i18n::{Language, set_language};
use rust_app_ui::transport::Transport;
use serde_json::Value;

/// Tests run in parallel inside one binary; the locale is global, so copy
/// assertions take this lock for the whole render.
static LOCALE_LOCK: Mutex<()> = Mutex::new(());

pub fn with_locale<T>(language: Language, f: impl FnOnce() -> T) -> T {
    let _guard = LOCALE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    set_language(language);
    f()
}

#[derive(Clone)]
pub struct FakeTransport {
    inner: Rc<RefCell<FakeState>>,
}

impl PartialEq for FakeTransport {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

struct FakeState {
    response: Result<Value, UiError>,
    calls: Vec<(String, Value)>,
}

impl FakeTransport {
    pub fn ok(value: Value) -> Self {
        Self::with_response(Ok(value))
    }

    pub fn err(payload: Value) -> Self {
        Self::with_response(Err(app_error(payload)))
    }

    fn with_response(response: Result<Value, UiError>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FakeState {
                response,
                calls: Vec::new(),
            })),
        }
    }

    pub fn calls(&self) -> Vec<(String, Value)> {
        self.inner.borrow().calls.clone()
    }

    pub fn last_command(&self) -> String {
        self.inner
            .borrow()
            .calls
            .last()
            .map(|(command, _)| command.clone())
            .unwrap_or_default()
    }

    pub fn last_args(&self) -> Value {
        self.inner
            .borrow()
            .calls
            .last()
            .map(|(_, args)| args.clone())
            .unwrap_or(Value::Null)
    }
}

#[async_trait(?Send)]
impl Transport for FakeTransport {
    async fn invoke(&self, command: &str, args: Value) -> Result<Value, UiError> {
        let mut state = self.inner.borrow_mut();
        state.calls.push((command.to_string(), args));
        state.response.clone()
    }
}

#[derive(Clone, Default)]
pub struct FakeToasts {
    items: Rc<RefCell<Vec<Toast>>>,
}

impl FakeToasts {
    pub fn items(&self) -> Vec<Toast> {
        self.items.borrow().clone()
    }
}

impl ToastSink for FakeToasts {
    fn push(&self, toast: Toast) {
        self.items.borrow_mut().push(toast);
    }
}
