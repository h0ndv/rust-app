use std::cell::RefCell;

use contracts::SettingsDto;
use dioxus::prelude::*;

use crate::api::Api;
use crate::components::{ErrorNote, LanguageSelect, Loading, Screen};
use crate::i18n::{Language, resolve_language, t, tn};
use crate::notifications::ensure_notification_permission;
use crate::state::{spawn_future, use_app};
use crate::theme::{normalize_theme, set_theme};

pub const THEMES: [(&str, &str); 3] = [
    ("system", "settings.themes.system"),
    ("light", "settings.themes.light"),
    ("dark", "settings.themes.dark"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsForm {
    pub sync_interval_minutes: u32,
    pub downloads_dir: String,
    pub theme: String,
    pub notifications: bool,
    pub language: Language,
}

impl From<SettingsDto> for SettingsForm {
    fn from(dto: SettingsDto) -> Self {
        Self {
            sync_interval_minutes: dto.sync_interval_minutes,
            downloads_dir: dto.downloads_dir,
            theme: normalize_theme(&dto.theme).as_str().to_string(),
            notifications: dto.notifications,
            language: resolve_language(&dto.language),
        }
    }
}

impl SettingsForm {
    pub fn to_dto(&self) -> SettingsDto {
        SettingsDto {
            sync_interval_minutes: self.sync_interval_minutes,
            downloads_dir: self.downloads_dir.clone(),
            theme: self.theme.clone(),
            notifications: self.notifications,
            language: self.language.code().to_string(),
        }
    }

    pub fn with_language(&self, language: Language) -> Self {
        Self {
            language,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsStatus {
    Saved,
    SaveFailed,
    ClearFailed,
    NotificationsDenied,
    Cleared(u64),
    DownloadsCleared(u64),
    Exported(String),
    ExportFailed,
    ClearDownloadsFailed,
}

impl SettingsStatus {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Saved => "settings.saved",
            Self::SaveFailed => "settings.saveFailed",
            Self::ClearFailed => "settings.clearFailed",
            Self::NotificationsDenied => "settings.notificationsDenied",
            Self::ExportFailed => "settings.exportFailed",
            Self::ClearDownloadsFailed => "settings.clearDownloadsFailed",
            Self::Cleared(_) => "settings.cleared",
            Self::DownloadsCleared(_) => "settings.downloadsCleared",
            Self::Exported(_) => "settings.exported",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Cleared(count) => tn("settings.cleared", count_i64(*count)),
            Self::DownloadsCleared(count) => tn("settings.downloadsCleared", count_i64(*count)),
            Self::Exported(path) => t("settings.exported").replace("%{path}", path),
            _ => t(self.key()),
        }
    }
}

fn count_i64(count: u64) -> i64 {
    i64::try_from(count).unwrap_or(i64::MAX)
}

thread_local! {
    /// A language change rebuilds the router, which remounts this screen. The
    /// saved status crosses that remount through this one-shot slot.
    static PENDING_STATUS: RefCell<Option<SettingsStatus>> = const { RefCell::new(None) };
}

fn remember_status(status: &SettingsStatus) {
    PENDING_STATUS.with(|slot| *slot.borrow_mut() = Some(status.clone()));
}

fn take_pending_status() -> Option<SettingsStatus> {
    PENDING_STATUS.with(|slot| slot.borrow_mut().take())
}

/// Persists the form and applies the three preferences. The caller updates the
/// context language signal, which rebuilds the tree with the new catalog.
pub async fn save_settings(api: &Api, form: &SettingsForm) -> SettingsStatus {
    if api.save_settings(form.to_dto()).await.is_err() {
        return SettingsStatus::SaveFailed;
    }

    crate::i18n::set_language(form.language);
    set_theme(&form.theme, api);

    if form.notifications && !ensure_notification_permission(api).await {
        return SettingsStatus::NotificationsDenied;
    }

    SettingsStatus::Saved
}

pub async fn export_local_data(api: &Api) -> SettingsStatus {
    match api.export_data().await {
        Ok(path) => SettingsStatus::Exported(path),
        Err(_) => SettingsStatus::ExportFailed,
    }
}

pub async fn clear_local_data(api: &Api) -> SettingsStatus {
    match api.clear_data().await {
        Ok(count) => SettingsStatus::Cleared(count),
        Err(_) => SettingsStatus::ClearFailed,
    }
}

pub async fn clear_downloaded_files(api: &Api) -> SettingsStatus {
    match api.clear_downloads().await {
        Ok(count) => SettingsStatus::DownloadsCleared(count),
        Err(_) => SettingsStatus::ClearDownloadsFailed,
    }
}

/// The webview asks before a destructive command. Native builds only run tests,
/// which never confirm.
pub fn confirm_clear(message: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        false
    }
}

#[component]
pub fn SettingsView(
    form: SettingsForm,
    #[props(default)] status: Option<SettingsStatus>,
    on_change: EventHandler<SettingsForm>,
    on_save: EventHandler<()>,
    on_export: EventHandler<()>,
    on_clear: EventHandler<()>,
    on_clear_downloads: EventHandler<()>,
    on_language: EventHandler<Language>,
) -> Element {
    let interval_form = form.clone();
    let dir_form = form.clone();
    let theme_form = form.clone();
    let notifications_form = form.clone();

    let input_class = "rounded-md border border-border bg-background px-3 py-2";

    rsx! {
        div { class: "flex max-w-md flex-col gap-4",
            label { class: "flex flex-col gap-1 text-sm",
                span { class: "font-medium", "{t(\"settings.language\")}" }
                LanguageSelect {
                    class: "rounded-md border border-border bg-background px-3 py-2",
                    aria_label: t("settings.language"),
                    value: form.language,
                    on_change: move |language| on_language.call(language),
                }
            }

            label { class: "flex flex-col gap-1 text-sm",
                span { class: "font-medium", "{t(\"settings.syncInterval\")}" }
                input {
                    r#type: "number",
                    min: "5",
                    class: input_class,
                    value: "{form.sync_interval_minutes}",
                    oninput: move |event| {
                        let minutes = event
                            .value()
                            .parse()
                            .unwrap_or(interval_form.sync_interval_minutes);
                        let mut next = interval_form.clone();
                        next.sync_interval_minutes = minutes;
                        on_change.call(next);
                    },
                }
            }

            label { class: "flex flex-col gap-1 text-sm",
                span { class: "font-medium", "{t(\"settings.downloadsDir\")}" }
                input {
                    r#type: "text",
                    class: input_class,
                    value: "{form.downloads_dir}",
                    oninput: move |event| {
                        let mut next = dir_form.clone();
                        next.downloads_dir = event.value();
                        on_change.call(next);
                    },
                }
            }

            label { class: "flex flex-col gap-1 text-sm",
                span { class: "font-medium", "{t(\"settings.theme\")}" }
                select {
                    class: input_class,
                    value: "{form.theme}",
                    onchange: move |event| {
                        let mut next = theme_form.clone();
                        next.theme = event.value();
                        on_change.call(next);
                    },
                    for (value, key) in THEMES {
                        option {
                            key: "{value}",
                            value,
                            selected: value == form.theme.as_str(),
                            "{t(key)}"
                        }
                    }
                }
            }

            label { class: "flex items-center gap-2 text-sm",
                input {
                    r#type: "checkbox",
                    checked: form.notifications,
                    onchange: move |event| {
                        let mut next = notifications_form.clone();
                        next.notifications = event.checked();
                        on_change.call(next);
                    },
                }
                span { class: "font-medium", "{t(\"settings.notifications\")}" }
            }

            div { class: "flex flex-wrap gap-2",
                button {
                    r#type: "button",
                    class: "rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground",
                    onclick: move |_| on_save.call(()),
                    "{t(\"settings.save\")}"
                }
            }

            section { class: "mt-4 flex flex-col gap-2 rounded-lg border border-border p-4",
                h3 { class: "text-sm font-semibold", "{t(\"settings.exportTitle\")}" }
                p { class: "text-xs text-muted-foreground", "{t(\"settings.exportBody\")}" }
                button {
                    r#type: "button",
                    class: "self-start rounded-md border border-border px-3 py-1 text-sm",
                    onclick: move |_| on_export.call(()),
                    "{t(\"settings.exportButton\")}"
                }
            }

            section { class: "mt-4 flex flex-col gap-2 rounded-lg border border-destructive/40 p-4",
                h3 { class: "text-sm font-semibold", "{t(\"settings.clearTitle\")}" }
                p { class: "text-xs text-muted-foreground", "{t(\"settings.clearBody\")}" }
                button {
                    r#type: "button",
                    class: "self-start rounded-md border border-destructive px-3 py-1 text-sm text-destructive",
                    onclick: move |_| on_clear.call(()),
                    "{t(\"settings.clearButton\")}"
                }
                button {
                    r#type: "button",
                    class: "self-start rounded-md border border-destructive px-3 py-1 text-sm text-destructive",
                    onclick: move |_| on_clear_downloads.call(()),
                    "{t(\"settings.clearDownloadsButton\")}"
                }
            }

            if let Some(status) = status {
                p { class: "text-sm text-muted-foreground", "{status.message()}" }
            }
        }
    }
}

#[component]
pub fn SettingsScreen() -> Element {
    let app = use_app();
    let resource_api = app.api.clone();
    let mut form = use_signal(|| None::<SettingsForm>);
    let mut status = use_signal(take_pending_status);

    let settings = use_resource(move || {
        let api = resource_api.clone();
        async move {
            let result = api.get_settings().await;
            if form.peek().is_none()
                && let Ok(dto) = &result
            {
                form.set(Some(SettingsForm::from(dto.clone())));
            }
            result
        }
    });

    let on_change = move |next: SettingsForm| form.set(Some(next));

    let on_language = move |language: Language| {
        if let Some(next) = form() {
            form.set(Some(next.with_language(language)));
        }
    };

    let on_save = {
        let api = app.api.clone();
        let context = app.clone();
        move |_| {
            let Some(current) = form() else {
                return;
            };
            let api = api.clone();
            let context = context.clone();
            spawn_future(async move {
                let result = save_settings(&api, &current).await;
                if result != SettingsStatus::SaveFailed {
                    if *context.language.peek() != current.language {
                        remember_status(&result);
                    }
                    context.set_language(current.language);
                }
                status.set(Some(result));
            });
        }
    };

    let on_export = {
        let api = app.api.clone();
        move |_| {
            let api = api.clone();
            spawn_future(async move {
                status.set(Some(export_local_data(&api).await));
            });
        }
    };

    let on_clear = {
        let api = app.api.clone();
        let mut data_version = app.data_version;
        move |_| {
            if !confirm_clear(&t("settings.clearConfirm")) {
                return;
            }

            let api = api.clone();
            spawn_future(async move {
                let result = clear_local_data(&api).await;
                if matches!(result, SettingsStatus::Cleared(_)) {
                    let next = (*data_version.peek()).wrapping_add(1);
                    data_version.set(next);
                }
                status.set(Some(result));
            });
        }
    };

    let on_clear_downloads = {
        let api = app.api.clone();
        let mut data_version = app.data_version;
        move |_| {
            if !confirm_clear(&t("settings.clearDownloadsConfirm")) {
                return;
            }

            let api = api.clone();
            spawn_future(async move {
                let result = clear_downloaded_files(&api).await;
                if matches!(result, SettingsStatus::DownloadsCleared(_)) {
                    let next = (*data_version.peek()).wrapping_add(1);
                    data_version.set(next);
                }
                status.set(Some(result));
            });
        }
    };

    let current = form();
    let subtitle = current.as_ref().map(|_| t("settings.subtitle"));

    let body = match (current, settings.cloned()) {
        (Some(form), _) => rsx! {
            SettingsView {
                form,
                status: status(),
                on_change,
                on_save,
                on_export,
                on_clear,
                on_clear_downloads,
                on_language,
            }
        },
        (None, Some(Err(error))) => rsx! { ErrorNote { kind: error.kind } },
        _ => rsx! { Loading {} },
    };

    rsx! {
        Screen { title: t("settings.title"), subtitle,
            {body}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SettingsStatus, remember_status, take_pending_status};

    #[test]
    fn a_pending_status_survives_exactly_one_remount() {
        remember_status(&SettingsStatus::Saved);

        assert_eq!(take_pending_status(), Some(SettingsStatus::Saved));
        assert_eq!(take_pending_status(), None);
    }
}
