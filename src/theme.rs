use std::cell::Cell;

use crate::api::Api;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

pub fn normalize_theme(value: &str) -> ThemePreference {
    match value {
        "light" => ThemePreference::Light,
        "dark" => ThemePreference::Dark,
        _ => ThemePreference::System,
    }
}

pub fn resolve_theme(value: &str, system_dark: bool) -> &'static str {
    match normalize_theme(value) {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
        ThemePreference::System if system_dark => "dark",
        ThemePreference::System => "light",
    }
}

thread_local! {
    static PREFERENCE: Cell<ThemePreference> = const { Cell::new(ThemePreference::System) };
}

#[cfg(target_arch = "wasm32")]
type ChangeListener = wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MediaQueryListEvent)>;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static API: std::cell::RefCell<Option<Api>> = const { std::cell::RefCell::new(None) };
    static MEDIA: std::cell::RefCell<Option<web_sys::MediaQueryList>> =
        const { std::cell::RefCell::new(None) };
    static LISTENER: std::cell::RefCell<Option<ChangeListener>> =
        const { std::cell::RefCell::new(None) };
}

pub fn init_theme(value: &str, api: &Api) {
    PREFERENCE.with(|preference| preference.set(normalize_theme(value)));

    #[cfg(target_arch = "wasm32")]
    {
        API.with(|slot| *slot.borrow_mut() = Some(api.clone()));
        listen_to_system();
        apply();
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = api;
}

pub fn set_theme(value: &str, api: &Api) {
    PREFERENCE.with(|preference| preference.set(normalize_theme(value)));

    #[cfg(target_arch = "wasm32")]
    {
        API.with(|slot| *slot.borrow_mut() = Some(api.clone()));
        apply();
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = api;
}

#[cfg(target_arch = "wasm32")]
fn listen_to_system() {
    use wasm_bindgen::JsCast;

    detach_listener();

    let Some(media) = web_sys::window().and_then(|window| {
        window
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
    }) else {
        return;
    };

    let listener = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MediaQueryListEvent)>::new(
        |_event: web_sys::MediaQueryListEvent| apply(),
    );

    if media
        .add_event_listener_with_callback("change", listener.as_ref().unchecked_ref())
        .is_ok()
    {
        MEDIA.with(|slot| *slot.borrow_mut() = Some(media));
        LISTENER.with(|slot| *slot.borrow_mut() = Some(listener));
    }
}

#[cfg(target_arch = "wasm32")]
fn detach_listener() {
    use wasm_bindgen::JsCast;

    let media = MEDIA.with(|slot| slot.borrow_mut().take());
    let listener = LISTENER.with(|slot| slot.borrow_mut().take());

    if let (Some(media), Some(listener)) = (media, listener) {
        let _ =
            media.remove_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
    }
}

#[cfg(target_arch = "wasm32")]
fn apply() {
    use wasm_bindgen::JsCast;

    let resolved = resolve_theme(
        PREFERENCE.with(|preference| preference.get().as_str()),
        system_prefers_dark(),
    );

    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        let _ = root
            .class_list()
            .toggle_with_force("dark", resolved == "dark");

        if let Ok(root) = root.dyn_into::<web_sys::HtmlElement>() {
            let _ = root.style().set_property("color-scheme", resolved);
        }
    }

    let api = API.with(|slot| slot.borrow().clone());
    if let Some(api) = api {
        let theme = resolved.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api.set_window_theme(&theme).await;
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn system_prefers_dark() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
        })
        .is_some_and(|media| media.matches())
}
