use crate::api::Api;
use crate::components::{Toast, ToastKind, ToastSink};
use crate::i18n::t;

pub async fn open_in_app(api: &Api, toasts: &dyn ToastSink, url: &str) {
    if let Err(error) = api.open_in_app(url).await {
        toasts.push(Toast::new(ToastKind::Error, t(error.key())));
    }
}
