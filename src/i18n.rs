#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Es,
}

impl Language {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
        }
    }

    pub const fn locale(self) -> &'static str {
        match self {
            Self::En => "en-US",
            Self::Es => "es-CL",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Español",
        }
    }
}

pub const LANGUAGES: [Language; 2] = [Language::En, Language::Es];

pub fn resolve_language(value: &str) -> Language {
    match value {
        "es" => Language::Es,
        _ => Language::En,
    }
}

pub fn set_language(language: Language) {
    rust_i18n::set_locale(language.code());

    #[cfg(target_arch = "wasm32")]
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        let _ = root.set_attribute("lang", language.code());
    }
}

pub fn t(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}

pub fn tn(key: &str, count: i64) -> String {
    let form = if count == 1 { "one" } else { "other" };
    let plural = format!("{key}.{form}");
    rust_i18n::t!(plural, count = count).to_string()
}
