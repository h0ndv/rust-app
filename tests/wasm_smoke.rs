//! Browser smoke test for the Dioxus runtime. It mounts a component that does
//! not touch Tauri, asserts the DOM renders and clicks a button to prove the
//! event handler updates a signal.
//!
//! It mounts a local `Counter`, not `App`: `App` builds a `TauriTransport` that
//! calls `invoke`, and a browser test has no `__TAURI__` bridge to answer it.
//! The isolated component still exercises the real Dioxus web renderer, DOM
//! mounting and event delegation.
//!
//! Run it with:
//! `wasm-pack test --headless --chrome --test wasm_smoke --features web`
#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const ROOT_ID: &str = "wasm-smoke-root";
const TIMEOUT_MS: f64 = 5_000.0;

#[component]
fn Counter() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        p { id: "counter-value", "Count: {count}" }
        button {
            id: "counter-button",
            r#type: "button",
            onclick: move |_| count += 1,
            "Increment"
        }
    }
}

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("window")
        .document()
        .expect("document")
}

fn now_ms() -> f64 {
    js_sys::Date::now()
}

async fn wait_for(selector: &str, deadline: f64) -> web_sys::Element {
    let mut backoff = 10;
    loop {
        if let Some(element) = document().query_selector(selector).expect("selector") {
            return element;
        }
        if now_ms() >= deadline {
            panic!("{selector} never mounted");
        }
        gloo_timers::future::TimeoutFuture::new(backoff).await;
        backoff = (backoff * 2).min(100);
    }
}

fn text_of(selector: &str) -> Option<String> {
    document()
        .query_selector(selector)
        .expect("selector")
        .and_then(|element| element.text_content())
}

#[wasm_bindgen_test]
async fn mounts_renders_text_and_updates_a_signal_on_click() {
    let mount = document().create_element("div").expect("create the root");
    mount.set_id(ROOT_ID);
    document()
        .body()
        .expect("body")
        .append_child(&mount)
        .expect("append the root");
    let root = mount.clone();

    let dom = VirtualDom::new(Counter);
    dioxus::web::launch::launch_virtual_dom(dom, dioxus::web::Config::new().rootelement(mount));

    let deadline = now_ms() + TIMEOUT_MS;
    let value = wait_for("#counter-value", deadline).await;
    assert_eq!(value.text_content().as_deref(), Some("Count: 0"));

    wait_for("#counter-button", deadline)
        .await
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button")
        .click();

    let mut backoff = 10;
    loop {
        if text_of("#counter-value").as_deref() == Some("Count: 1") {
            break;
        }
        if now_ms() >= deadline {
            panic!(
                "the click did not update the signal, text was {:?}",
                text_of("#counter-value")
            );
        }
        gloo_timers::future::TimeoutFuture::new(backoff).await;
        backoff = (backoff * 2).min(100);
    }

    root.remove();
}
