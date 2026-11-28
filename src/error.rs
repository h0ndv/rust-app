use contracts::AppErrorPayload;

pub use contracts::ErrorKind;

/// Localization key for each [`ErrorKind`], in declaration order. `ErrorKind`
/// is fieldless, so `kind as usize` indexes it; the test in `tests/api.rs`
/// pins every slot to its key.
pub const ERROR_KEY: [&str; 7] = [
    "errors.network",
    "errors.unauthorized",
    "errors.forbidden",
    "errors.parse",
    "errors.not_found",
    "errors.rate_limited",
    "errors.internal",
];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct UiError {
    pub kind: ErrorKind,
    pub message: String,
}

impl UiError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn key(&self) -> &'static str {
        ERROR_KEY[self.kind as usize]
    }
}

impl From<AppErrorPayload> for UiError {
    fn from(payload: AppErrorPayload) -> Self {
        Self::new(payload.kind, payload.message)
    }
}

/// Turns a rejected command payload into a [`UiError`]. A payload that does not
/// match `AppErrorPayload` (an unrecognized kind, a plain string) becomes an
/// internal error, keeping the original message when there is one.
pub fn app_error(payload: serde_json::Value) -> UiError {
    let message = payload
        .get("message")
        .and_then(serde_json::Value::as_str)
        .or_else(|| payload.as_str())
        .unwrap_or("unexpected command failure")
        .to_string();

    match serde_json::from_value::<AppErrorPayload>(payload) {
        Ok(payload) => payload.into(),
        Err(_) => UiError::internal(message),
    }
}
