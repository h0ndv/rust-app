#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use async_trait::async_trait;
use common::FakeTransport;
use contracts::{SessionState, SettingsDto};
use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use rust_app_ui::api::Api;
use rust_app_ui::components::ToastStore;
use rust_app_ui::error::UiError;
use rust_app_ui::features::settings::{
    SettingsForm, SettingsScreen, SettingsStatus, SettingsView, clear_downloaded_files,
    clear_local_data, confirm_clear, export_local_data, save_settings,
};
use rust_app_ui::i18n::Language;
use rust_app_ui::state::{AppContext, SyncView};
use rust_app_ui::transport::Transport;
use serde_json::{Value, json};

fn settings() -> SettingsDto {
    SettingsDto {
        sync_interval_minutes: 30,
        downloads_dir: "C:\\Downloads".into(),
        theme: "system".into(),
        notifications: false,
        language: "en".into(),
    }
}

fn form() -> SettingsForm {
    SettingsForm::from(settings())
}

fn block_on<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("a current-thread runtime")
        .block_on(future)
}

#[component]
fn ViewHarness(form: SettingsForm, #[props(default)] status: Option<SettingsStatus>) -> Element {
    rsx! {
        SettingsView {
            form,
            status,
            on_change: |_| {},
            on_save: |_| {},
            on_export: |_| {},
            on_clear: |_| {},
            on_clear_downloads: |_| {},
            on_language: |_| {},
        }
    }
}

fn render_view(form: SettingsForm, status: Option<SettingsStatus>) -> String {
    dioxus_ssr::render_element(rsx! {
        ViewHarness { form, status }
    })
}

#[derive(Props, Clone)]
struct ScreenProps {
    transport: FakeTransport,
}

impl PartialEq for ScreenProps {
    fn eq(&self, other: &Self) -> bool {
        self.transport == other.transport
    }
}

fn screen(props: ScreenProps) -> Element {
    let api = use_hook(|| Api::new(Rc::new(props.transport.clone())));
    let toasts = use_hook(ToastStore::new);
    let sync = use_signal(SyncView::default);
    let session = use_signal(|| None::<SessionState>);
    let data_version = use_signal(|| 0u64);
    let language = use_signal(|| Language::En);
    use_context_provider(|| AppContext {
        api,
        sync,
        session,
        data_version,
        toasts,
        language,
    });

    rsx! {
        SettingsScreen {}
    }
}

fn mounted(transport: FakeTransport) -> VirtualDom {
    let mut vdom = VirtualDom::new_with_props(screen, ScreenProps { transport });
    vdom.rebuild(&mut NoOpMutations);
    vdom.render_immediate(&mut NoOpMutations);
    vdom.render_immediate(&mut NoOpMutations);

    vdom
}

#[derive(Clone)]
struct RoutedTransport {
    routes: Rc<Vec<(&'static str, Value)>>,
    calls: Rc<RefCell<Vec<(String, Value)>>>,
}

impl RoutedTransport {
    fn new(routes: Vec<(&'static str, Value)>) -> Self {
        Self {
            routes: Rc::new(routes),
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn commands(&self) -> Vec<String> {
        self.calls
            .borrow()
            .iter()
            .map(|(command, _)| command.clone())
            .collect()
    }
}

#[async_trait(?Send)]
impl Transport for RoutedTransport {
    async fn invoke(&self, command: &str, args: Value) -> Result<Value, UiError> {
        self.calls.borrow_mut().push((command.to_string(), args));

        self.routes
            .iter()
            .find(|(name, _)| *name == command)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| UiError::internal(format!("no route for {command}")))
    }
}

#[test]
fn form_round_trips_the_settings_dto() {
    let form = SettingsForm::from(settings());

    assert_eq!(form.to_dto(), settings());
}

#[test]
fn form_normalizes_unknown_preferences() {
    let form = SettingsForm::from(SettingsDto {
        theme: "midnight".into(),
        language: "fr".into(),
        ..settings()
    });

    assert_eq!(form.theme, "system");
    assert_eq!(form.language, Language::En);
    assert_eq!(form.to_dto().theme, "system");
    assert_eq!(form.to_dto().language, "en");
}

#[test]
fn changing_the_language_keeps_the_rest_of_the_form() {
    let next = form().with_language(Language::Es);

    assert_eq!(next.language, Language::Es);
    assert_eq!(next.to_dto().language, "es");
    assert_eq!(next.sync_interval_minutes, 30);
    assert_eq!(next.downloads_dir, "C:\\Downloads");
    assert_eq!(next.theme, "system");
}

#[test]
fn denial_status_maps_to_the_localized_key() {
    assert_eq!(
        SettingsStatus::NotificationsDenied.key(),
        "settings.notificationsDenied"
    );
}

#[test]
fn every_status_maps_to_its_catalog_key() {
    assert_eq!(SettingsStatus::Saved.key(), "settings.saved");
    assert_eq!(SettingsStatus::SaveFailed.key(), "settings.saveFailed");
    assert_eq!(SettingsStatus::ClearFailed.key(), "settings.clearFailed");
    assert_eq!(SettingsStatus::ExportFailed.key(), "settings.exportFailed");
    assert_eq!(
        SettingsStatus::ClearDownloadsFailed.key(),
        "settings.clearDownloadsFailed"
    );
    assert_eq!(SettingsStatus::Cleared(3).key(), "settings.cleared");
    assert_eq!(
        SettingsStatus::DownloadsCleared(2).key(),
        "settings.downloadsCleared"
    );
    assert_eq!(
        SettingsStatus::Exported("path".into()).key(),
        "settings.exported"
    );
}

#[test]
fn counted_and_path_statuses_interpolate_their_values() {
    common::with_locale(Language::En, || {
        assert_eq!(SettingsStatus::Cleared(1).message(), "1 row deleted.");
        assert_eq!(SettingsStatus::Cleared(3).message(), "3 rows deleted.");
        assert_eq!(
            SettingsStatus::DownloadsCleared(1).message(),
            "1 file deleted."
        );
        assert_eq!(
            SettingsStatus::DownloadsCleared(2).message(),
            "2 files deleted."
        );
        assert_eq!(
            SettingsStatus::Exported("C:\\Downloads\\rust-app-export.json".into()).message(),
            "Export saved to C:\\Downloads\\rust-app-export.json"
        );
        assert_eq!(SettingsStatus::Saved.message(), "Settings saved.");
        assert_eq!(
            SettingsStatus::NotificationsDenied.message(),
            "Notifications are blocked by the system."
        );
    });
}

#[test]
fn the_form_shows_the_saved_values_and_every_control() {
    let html = common::with_locale(Language::En, || render_view(form(), None));

    assert!(html.contains("aria-label=\"Language\""), "html was {html}");
    assert!(html.contains("value=\"en\""), "html was {html}");
    assert!(html.contains("Sync interval (minutes)"), "html was {html}");
    assert!(html.contains("value=\"30\""), "html was {html}");
    assert!(html.contains("Downloads folder"), "html was {html}");
    assert!(html.contains("C:\\Downloads"), "html was {html}");
    assert!(html.contains(">Theme<"), "html was {html}");
    assert!(html.contains("value=\"system\""), "html was {html}");
    assert!(html.contains(">System<"), "html was {html}");
    assert!(html.contains(">Light<"), "html was {html}");
    assert!(html.contains(">Dark<"), "html was {html}");
    assert!(
        html.contains("Notify me about due dates"),
        "html was {html}"
    );
    assert!(html.contains(">Save<"), "html was {html}");
    assert!(html.contains("Export data"), "html was {html}");
    assert!(html.contains("Delete data"), "html was {html}");
    assert!(html.contains("Clear downloaded files"), "html was {html}");
}

#[test]
fn the_selects_mark_the_saved_choices() {
    let mut form = form();
    form.language = Language::Es;
    form.theme = "dark".into();

    let html = common::with_locale(Language::En, || render_view(form, None));

    assert_eq!(html.matches("selected=true").count(), 2, "html was {html}");
    assert!(html.contains("value=\"es\""), "html was {html}");
    assert!(html.contains("value=\"dark\""), "html was {html}");
}

#[test]
fn the_form_reflects_a_dark_theme_and_enabled_notifications() {
    let mut form = form();
    form.theme = "dark".into();
    form.notifications = true;

    let html = common::with_locale(Language::En, || render_view(form, None));

    assert!(html.contains("value=\"dark\""), "html was {html}");
    assert!(html.contains("checked"), "html was {html}");
}

#[test]
fn the_form_copy_follows_the_locale() {
    let html = common::with_locale(Language::Es, || render_view(form(), None));

    assert!(html.contains(">Idioma<"), "html was {html}");
    assert!(
        html.contains("Intervalo de sincronización (minutos)"),
        "html was {html}"
    );
    assert!(html.contains("Carpeta de descargas"), "html was {html}");
    assert!(html.contains(">Tema<"), "html was {html}");
    assert!(html.contains(">Sistema<"), "html was {html}");
    assert!(html.contains("Avisar de vencimientos"), "html was {html}");
    assert!(html.contains(">Guardar<"), "html was {html}");
    assert!(html.contains("Exportar datos"), "html was {html}");
    assert!(html.contains("Borrar datos"), "html was {html}");
    assert!(
        html.contains("Borrar archivos descargados"),
        "html was {html}"
    );
}

#[test]
fn the_status_line_renders_the_current_status() {
    let saved = common::with_locale(Language::En, || {
        render_view(form(), Some(SettingsStatus::Saved))
    });
    assert!(saved.contains("Settings saved."), "html was {saved}");

    let exported = common::with_locale(Language::En, || {
        render_view(
            form(),
            Some(SettingsStatus::Exported(
                "C:\\Downloads\\rust-app-export.json".into(),
            )),
        )
    });
    assert!(
        exported.contains("Export saved to C:\\Downloads\\rust-app-export.json"),
        "html was {exported}"
    );

    let cleared = common::with_locale(Language::En, || {
        render_view(form(), Some(SettingsStatus::Cleared(2)))
    });
    assert!(cleared.contains("2 rows deleted."), "html was {cleared}");
}

#[test]
fn the_screen_fetches_the_settings_and_fills_the_form() {
    let fake = FakeTransport::ok(serde_json::to_value(settings()).expect("settings payload"));

    let html = common::with_locale(Language::En, || {
        let vdom = mounted(fake.clone());
        dioxus_ssr::render(&vdom)
    });

    assert_eq!(
        fake.calls(),
        vec![("get_settings".to_string(), json!({}))],
        "html was {html}"
    );
    assert!(html.contains("Everything is stored on this computer."));
    assert!(html.contains("value=\"30\""), "html was {html}");
    assert!(html.contains("C:\\Downloads"), "html was {html}");
    assert!(html.contains(">Save<"), "html was {html}");
}

#[test]
fn the_screen_reports_a_failed_fetch() {
    let fake = FakeTransport::err(json!({
        "kind": "internal",
        "message": "the database is locked"
    }));

    let html = common::with_locale(Language::En, || {
        let vdom = mounted(fake);
        dioxus_ssr::render(&vdom)
    });

    assert!(html.contains("Something went wrong."), "html was {html}");
}

#[test]
fn saving_sends_the_whole_dto_and_switches_the_locale() {
    let fake = FakeTransport::ok(Value::Null);
    let api = Api::new(Rc::new(fake.clone()));
    let mut form = form();
    form.sync_interval_minutes = 45;
    form.downloads_dir = "D:\\Moodle".into();
    form.theme = "dark".into();
    form.notifications = false;
    form.language = Language::Es;

    common::with_locale(Language::En, || {
        let status = block_on(save_settings(&api, &form));

        assert_eq!(status, SettingsStatus::Saved);
        assert_eq!(fake.last_command(), "save_settings");
        assert_eq!(
            fake.last_args(),
            json!({
                "settings": {
                    "sync_interval_minutes": 45,
                    "downloads_dir": "D:\\Moodle",
                    "theme": "dark",
                    "notifications": false,
                    "language": "es"
                }
            })
        );
        assert_eq!(&*rust_i18n::locale(), "es");
    });
}

#[test]
fn a_rejected_save_reports_failure_without_switching_the_locale() {
    let fake = FakeTransport::err(json!({
        "kind": "internal",
        "message": "could not write settings"
    }));
    let api = Api::new(Rc::new(fake));
    let mut form = form();
    form.language = Language::Es;

    common::with_locale(Language::En, || {
        let status = block_on(save_settings(&api, &form));

        assert_eq!(status, SettingsStatus::SaveFailed);
        assert_eq!(&*rust_i18n::locale(), "en");
    });
}

#[test]
fn a_denied_notification_permission_reports_its_status() {
    let transport = RoutedTransport::new(vec![
        ("save_settings", Value::Null),
        ("notification_permission", json!(false)),
        ("request_notification_permission", json!(false)),
    ]);
    let api = Api::new(Rc::new(transport.clone()));
    let mut form = form();
    form.notifications = true;

    let status = block_on(save_settings(&api, &form));

    assert_eq!(status, SettingsStatus::NotificationsDenied);
    assert_eq!(
        transport.commands(),
        vec![
            "save_settings",
            "notification_permission",
            "request_notification_permission"
        ]
    );
}

#[test]
fn a_granted_notification_permission_saves_without_asking_again() {
    let transport = RoutedTransport::new(vec![
        ("save_settings", Value::Null),
        ("notification_permission", json!(true)),
    ]);
    let api = Api::new(Rc::new(transport.clone()));
    let mut form = form();
    form.notifications = true;

    let status = block_on(save_settings(&api, &form));

    assert_eq!(status, SettingsStatus::Saved);
    assert_eq!(
        transport.commands(),
        vec!["save_settings", "notification_permission"]
    );
}

#[tokio::test]
async fn export_reports_the_path() {
    let fake = FakeTransport::ok(json!("C:\\Downloads\\rust-app-export.json"));
    let api = Api::new(Rc::new(fake.clone()));

    let status = export_local_data(&api).await;

    assert_eq!(
        status,
        SettingsStatus::Exported("C:\\Downloads\\rust-app-export.json".into())
    );
    assert_eq!(fake.last_command(), "export_data");
}

#[tokio::test]
async fn a_rejected_export_reports_failure() {
    let fake = FakeTransport::err(json!({
        "kind": "internal",
        "message": "the disk is full"
    }));
    let api = Api::new(Rc::new(fake));

    assert_eq!(export_local_data(&api).await, SettingsStatus::ExportFailed);
}

#[tokio::test]
async fn clearing_data_reports_the_deleted_count() {
    let fake = FakeTransport::ok(json!(42));
    let api = Api::new(Rc::new(fake.clone()));

    let status = clear_local_data(&api).await;

    assert_eq!(status, SettingsStatus::Cleared(42));
    assert_eq!(fake.last_command(), "clear_data");
}

#[tokio::test]
async fn a_rejected_clear_reports_failure() {
    let fake = FakeTransport::err(json!({
        "kind": "internal",
        "message": "the database is locked"
    }));
    let api = Api::new(Rc::new(fake));

    assert_eq!(clear_local_data(&api).await, SettingsStatus::ClearFailed);
}

#[tokio::test]
async fn clearing_downloads_reports_the_deleted_count() {
    let fake = FakeTransport::ok(json!(2));
    let api = Api::new(Rc::new(fake.clone()));

    let status = clear_downloaded_files(&api).await;

    assert_eq!(status, SettingsStatus::DownloadsCleared(2));
    assert_eq!(fake.last_command(), "clear_downloads");
}

#[tokio::test]
async fn a_rejected_clear_downloads_reports_failure() {
    let fake = FakeTransport::err(json!({
        "kind": "internal",
        "message": "the downloads folder is locked"
    }));
    let api = Api::new(Rc::new(fake));

    assert_eq!(
        clear_downloaded_files(&api).await,
        SettingsStatus::ClearDownloadsFailed
    );
}

#[test]
fn destructive_actions_stay_unconfirmed_outside_the_webview() {
    assert!(!confirm_clear("Local data will be deleted. Continue?"));
}
