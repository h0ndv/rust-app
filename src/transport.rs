use async_trait::async_trait;
use serde_json::Value;

use crate::error::UiError;

#[async_trait(?Send)]
pub trait Transport {
    async fn invoke(&self, command: &str, args: Value) -> Result<Value, UiError>;
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedTransport;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait(?Send)]
impl Transport for UnsupportedTransport {
    async fn invoke(&self, _command: &str, _args: Value) -> Result<Value, UiError> {
        Err(no_transport())
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default, Clone, Copy)]
pub struct TauriTransport;

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl Transport for TauriTransport {
    async fn invoke(&self, command: &str, args: Value) -> Result<Value, UiError> {
        invoke_tauri(command, args).await
    }
}

fn no_transport() -> UiError {
    UiError::internal("no tauri transport")
}

#[cfg(target_arch = "wasm32")]
async fn invoke_tauri(command: &str, args: Value) -> Result<Value, UiError> {
    use js_sys::wasm_bindgen::{JsCast, JsValue};

    let global = js_sys::global();
    let tauri = js_sys::Reflect::get(&global, &JsValue::from_str("__TAURI__"))
        .map_err(|_| no_transport())?;
    if tauri.is_undefined() || tauri.is_null() {
        return Err(no_transport());
    }

    let core =
        js_sys::Reflect::get(&tauri, &JsValue::from_str("core")).map_err(|_| no_transport())?;
    let invoke =
        js_sys::Reflect::get(&core, &JsValue::from_str("invoke")).map_err(|_| no_transport())?;
    let invoke: js_sys::Function = invoke.dyn_into().map_err(|_| no_transport())?;

    // The default serializer emits ES maps, which Tauri cannot read as arguments.
    let payload =
        serde::Serialize::serialize(&args, &serde_wasm_bindgen::Serializer::json_compatible())
            .map_err(|error| UiError::internal(error.to_string()))?;

    let promise = invoke
        .call2(&JsValue::UNDEFINED, &JsValue::from_str(command), &payload)
        .map_err(rejection)?;
    let promise: js_sys::Promise = promise
        .dyn_into()
        .map_err(|_| UiError::internal("the tauri bridge did not return a promise"))?;

    let value = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(rejection)?;

    if value.is_undefined() || value.is_null() {
        return Ok(Value::Null);
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| UiError::internal(error.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn rejection(error: js_sys::wasm_bindgen::JsValue) -> UiError {
    if let Some(message) = error.as_string() {
        return UiError::internal(message);
    }

    match serde_wasm_bindgen::from_value::<Value>(error) {
        Ok(payload) => crate::error::app_error(payload),
        Err(_) => UiError::internal("the command was rejected"),
    }
}
