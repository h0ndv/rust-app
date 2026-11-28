use dioxus::prelude::*;

use crate::i18n::{LANGUAGES, Language, resolve_language};

#[component]
pub fn LanguageSelect(
    value: Language,
    on_change: EventHandler<Language>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let current = value;

    rsx! {
        select {
            id: id,
            class: "h-9 rounded-md border border-input bg-background px-2 text-sm {class}",
            aria_label: aria_label,
            value: current.code(),
            onchange: move |event| on_change.call(resolve_language(&event.value())),
            for language in LANGUAGES {
                option {
                    key: "{language.code()}",
                    value: language.code(),
                    selected: language == current,
                    "{language.label()}"
                }
            }
        }
    }
}
