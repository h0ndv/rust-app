#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::any::Any;
use std::rc::Rc;

use common::FakeTransport;
use contracts::{CourseDetailDto, CourseGroupDto, SessionState};
use dioxus::core::{AttributeValue, ElementId, Event, NoOpMutations, Template, WriteMutations};
use dioxus::history::{History, MemoryHistory};
use dioxus::html::{
    PlatformEventData, SerializedHtmlEventConverter, SerializedMouseData, set_event_converter,
};
use dioxus::prelude::*;
use dioxus::router::components::HistoryProvider;
use rust_app_ui::api::Api;
use rust_app_ui::components::ToastStore;
use rust_app_ui::features::course_detail::CourseDetailScreen;
use rust_app_ui::i18n::Language;
use rust_app_ui::state::{AppContext, SyncView};
use serde_json::json;

fn detail() -> CourseDetailDto {
    CourseDetailDto {
        group: CourseGroupDto {
            id: "g1".into(),
            code: "IGR-001".into(),
            name: "Mineria de Datos".into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Mirrors the production route pattern without the shell layout, so the screen
/// can mount with a memory history and a fake transport.
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum TestRoute {
    #[route("/ramos", CoursesStub)]
    Courses {},
    #[route("/ramos/:group_id?:focus", RoutedCourseDetail)]
    CourseDetail { group_id: String, focus: Option<String> },
}

#[component]
fn CoursesStub() -> Element {
    rsx! {
        div { "courses screen" }
    }
}

#[component]
fn RoutedCourseDetail(group_id: String, focus: Option<String>) -> Element {
    rsx! {
        CourseDetailScreen { group_id, focus }
    }
}

#[derive(Props, Clone)]
struct HarnessProps {
    transport: FakeTransport,
    history: Rc<MemoryHistory>,
}

impl PartialEq for HarnessProps {
    fn eq(&self, other: &Self) -> bool {
        self.transport == other.transport && Rc::ptr_eq(&self.history, &other.history)
    }
}

fn harness(props: HarnessProps) -> Element {
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

    let history = props.history.clone();

    rsx! {
        HistoryProvider {
            history: move |_| history.clone() as Rc<dyn History>,
            Router::<TestRoute> {}
        }
    }
}

#[derive(Default)]
struct Mutations {
    clicks: Vec<ElementId>,
}

impl WriteMutations for Mutations {
    fn append_children(&mut self, _id: ElementId, _m: usize) {}
    fn assign_node_id(&mut self, _path: &'static [u8], _id: ElementId) {}
    fn create_placeholder(&mut self, _id: ElementId) {}
    fn create_text_node(&mut self, _value: &str, _id: ElementId) {}
    fn load_template(&mut self, _template: Template, _index: usize, _id: ElementId) {}
    fn replace_node_with(&mut self, _id: ElementId, _m: usize) {}
    fn replace_placeholder_with_nodes(&mut self, _path: &'static [u8], _m: usize) {}
    fn insert_nodes_after(&mut self, _id: ElementId, _m: usize) {}
    fn insert_nodes_before(&mut self, _id: ElementId, _m: usize) {}
    fn set_attribute(
        &mut self,
        _name: &'static str,
        _ns: Option<&'static str>,
        _value: &AttributeValue,
        _id: ElementId,
    ) {
    }
    fn set_node_text(&mut self, _value: &str, _id: ElementId) {}
    fn create_event_listener(&mut self, name: &'static str, id: ElementId) {
        if name == "click" {
            self.clicks.push(id);
        }
    }
    fn remove_event_listener(&mut self, _name: &'static str, _id: ElementId) {}
    fn remove_node(&mut self, _id: ElementId) {}
    fn push_root(&mut self, _id: ElementId) {}
}

fn mounted(transport: FakeTransport) -> (VirtualDom, Rc<MemoryHistory>, Mutations) {
    let history = Rc::new(MemoryHistory::default());
    history.push("/ramos".to_string());
    history.push("/ramos/g1".to_string());

    let mut vdom = VirtualDom::new_with_props(
        harness,
        HarnessProps {
            transport,
            history: history.clone(),
        },
    );
    let mut mutations = Mutations::default();
    vdom.rebuild(&mut mutations);
    vdom.render_immediate(&mut mutations);

    (vdom, history, mutations)
}

#[test]
fn the_route_group_reaches_the_fetch() {
    let fake = FakeTransport::ok(serde_json::to_value(detail()).expect("a detail payload"));
    let (vdom, _history, _mutations) = mounted(fake.clone());

    assert_eq!(
        fake.calls(),
        vec![("get_course".to_string(), json!({ "groupId": "g1" }))],
        "html was {}",
        dioxus_ssr::render(&vdom)
    );
    let html = dioxus_ssr::render(&vdom);
    assert!(html.contains("Mineria de Datos"), "html was {html}");
}

#[test]
fn back_returns_to_the_previous_route() {
    let fake = FakeTransport::ok(serde_json::to_value(detail()).expect("a detail payload"));
    let (mut vdom, history, mutations) = mounted(fake);

    assert_eq!(history.current_route(), "/ramos/g1?");
    let back = mutations.clicks.first().copied().expect("a click listener");

    static CONVERTER: std::sync::Once = std::sync::Once::new();
    CONVERTER.call_once(|| set_event_converter(Box::new(SerializedHtmlEventConverter)));

    let data = PlatformEventData::new(Box::new(SerializedMouseData::default()));
    let runtime = vdom.runtime();
    runtime.handle_event("click", Event::<dyn Any>::new(Rc::new(data), true), back);
    vdom.render_immediate(&mut NoOpMutations);

    assert_eq!(history.current_route(), "/ramos");
    let html = dioxus_ssr::render(&vdom);
    assert!(html.contains("courses screen"), "html was {html}");
}
