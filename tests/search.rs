#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::any::Any;
use std::rc::Rc;

use common::FakeTransport;
use contracts::{SearchHitDto, SearchResultsDto, SessionState};
use dioxus::core::{AttributeValue, ElementId, Event, NoOpMutations, Template, WriteMutations};
use dioxus::history::{History, MemoryHistory};
use dioxus::html::{
    PlatformEventData, SerializedFormData, SerializedHtmlEventConverter, set_event_converter,
};
use dioxus::prelude::*;
use dioxus::router::components::HistoryProvider;
use rust_app_ui::api::Api;
use rust_app_ui::components::ToastStore;
use rust_app_ui::features::search::{
    SearchResultsView, SearchScreen, normalize_query, search_link, search_route,
};
use rust_app_ui::i18n::Language;
use rust_app_ui::router::Route;
use rust_app_ui::state::{AppContext, SyncView};
use serde_json::{Value, json};

fn hit(kind: &str) -> SearchHitDto {
    SearchHitDto {
        kind: kind.into(),
        id: "h1".into(),
        title: "Lab 1".into(),
        body: String::new(),
        group_id: Some("g1".into()),
    }
}

fn orphan(mut item: SearchHitDto) -> SearchHitDto {
    item.group_id = None;
    item
}

fn results(items: Vec<SearchHitDto>) -> SearchResultsDto {
    SearchResultsDto {
        query: "lab".into(),
        total: items.len(),
        items,
    }
}

fn render_results(data: SearchResultsDto) -> String {
    dioxus_ssr::render_element(rsx! {
        SearchResultsView { results: data }
    })
}

#[test]
fn normalize_query_trims_the_input() {
    assert_eq!(normalize_query("  minería  "), "minería");
    assert_eq!(normalize_query("\t\n"), "");
    assert_eq!(normalize_query("minería"), "minería");
}

#[test]
fn search_link_keeps_the_focus_for_non_announcements() {
    assert_eq!(search_link(&hit("file")), "/ramos/g1?focus=h1");
    assert_eq!(search_link(&hit("activity")), "/ramos/g1?focus=h1");
}

#[test]
fn announcements_link_without_a_focus() {
    assert_eq!(search_link(&hit("announcement")), "/ramos/g1");
}

#[test]
fn a_hit_without_a_course_links_to_the_course_list() {
    assert_eq!(search_link(&orphan(hit("file"))), "/ramos");
}

#[test]
fn search_route_targets_the_detail_screen_with_the_hit() {
    assert_eq!(
        search_route(&hit("activity")),
        Route::CourseDetail {
            group_id: "g1".into(),
            focus: Some("h1".into()),
        }
    );
    assert_eq!(
        search_route(&hit("announcement")),
        Route::CourseDetail {
            group_id: "g1".into(),
            focus: None,
        }
    );
    assert_eq!(search_route(&orphan(hit("file"))), Route::Courses {});
}

#[test]
fn a_hit_shows_the_kind_badge_title_and_body() {
    let mut item = orphan(hit("activity"));
    item.body = "Guía de laboratorio".into();

    let html = common::with_locale(Language::En, || render_results(results(vec![item])));

    assert!(html.contains(">Activity<"), "html was {html}");
    assert!(html.contains("Lab 1"), "html was {html}");
    assert!(html.contains("Guía de laboratorio"), "html was {html}");
    assert!(!html.contains("<a "), "html was {html}");
}

#[test]
fn a_hit_without_a_body_omits_the_paragraph() {
    let html = common::with_locale(Language::En, || {
        render_results(results(vec![orphan(hit("file"))]))
    });

    assert!(html.contains(">File<"), "html was {html}");
    assert!(
        !html.contains("text-xs text-muted-foreground"),
        "html was {html}"
    );
}

#[test]
fn zero_results_interpolate_the_query() {
    let empty = SearchResultsDto {
        query: "minería".into(),
        total: 0,
        items: Vec::new(),
    };

    let html = common::with_locale(Language::En, || render_results(empty));

    assert!(
        html.contains("No results for “minería”."),
        "html was {html}"
    );
}

#[test]
fn the_result_copy_follows_the_locale() {
    let mut item = orphan(hit("file"));
    item.body = "Guía".into();
    let empty = SearchResultsDto {
        query: "lab".into(),
        total: 0,
        items: Vec::new(),
    };

    common::with_locale(Language::Es, || {
        let html = render_results(results(vec![item]));
        assert!(html.contains(">Archivo<"), "html was {html}");

        let note = render_results(empty);
        assert!(
            note.contains("Sin resultados para “lab”."),
            "html was {note}"
        );
    });
}

#[component]
fn ResultsPage() -> Element {
    let results = use_context::<SearchResultsDto>();

    rsx! {
        SearchResultsView { results }
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum ResultsRoute {
    #[route("/")]
    ResultsPage {},
}

#[derive(Props, Clone)]
struct RoutedProps {
    results: SearchResultsDto,
    history: Rc<MemoryHistory>,
}

impl PartialEq for RoutedProps {
    fn eq(&self, other: &Self) -> bool {
        self.results == other.results && Rc::ptr_eq(&self.history, &other.history)
    }
}

fn routed_harness(props: RoutedProps) -> Element {
    use_context_provider(|| props.results.clone());
    let history = props.history.clone();

    rsx! {
        HistoryProvider {
            history: move |_| history.clone() as Rc<dyn History>,
            Router::<ResultsRoute> {}
        }
    }
}

fn render_routed(results: SearchResultsDto) -> String {
    let history = Rc::new(MemoryHistory::default());
    let mut vdom = VirtualDom::new_with_props(routed_harness, RoutedProps { results, history });
    vdom.rebuild(&mut NoOpMutations);
    vdom.render_immediate(&mut NoOpMutations);

    dioxus_ssr::render(&vdom)
}

#[test]
fn a_grouped_hit_links_to_its_course_and_focus() {
    let html = common::with_locale(Language::En, || {
        render_routed(results(vec![hit("activity")]))
    });

    assert!(
        html.contains("href=\"/ramos/g1?focus=h1\""),
        "html was {html}"
    );
}

#[test]
fn an_announcement_link_carries_no_focus() {
    let html = common::with_locale(Language::En, || {
        render_routed(results(vec![hit("announcement")]))
    });

    // The router Link keeps the empty query marker that `search_link` strips.
    assert!(html.contains("href=\"/ramos/g1?\""), "html was {html}");
    assert!(!html.contains("focus="), "html was {html}");
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
        SearchScreen {}
    }
}

#[derive(Default)]
struct Listeners {
    input: Option<ElementId>,
    submit: Option<ElementId>,
}

impl WriteMutations for Listeners {
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
        match name {
            "input" => self.input = Some(id),
            "submit" => self.submit = Some(id),
            _ => {}
        }
    }
    fn remove_event_listener(&mut self, _name: &'static str, _id: ElementId) {}
    fn remove_node(&mut self, _id: ElementId) {}
    fn push_root(&mut self, _id: ElementId) {}
}

fn mounted(transport: FakeTransport) -> (VirtualDom, Listeners) {
    let mut vdom = VirtualDom::new_with_props(screen, ScreenProps { transport });
    let mut listeners = Listeners::default();
    vdom.rebuild(&mut listeners);
    vdom.render_immediate(&mut NoOpMutations);

    (vdom, listeners)
}

fn form_event(value: &str) -> Event<dyn Any> {
    static CONVERTER: std::sync::Once = std::sync::Once::new();
    CONVERTER.call_once(|| set_event_converter(Box::new(SerializedHtmlEventConverter)));

    let data = PlatformEventData::new(Box::new(SerializedFormData::new(
        value.to_string(),
        Vec::new(),
    )));

    Event::<dyn Any>::new(Rc::new(data), true)
}

fn search_payload() -> Value {
    serde_json::to_value(results(vec![orphan(hit("activity"))])).expect("a search payload")
}

#[test]
fn an_empty_query_shows_the_prompt_and_does_not_fetch() {
    let fake = FakeTransport::ok(search_payload());

    let html = common::with_locale(Language::En, || {
        let (vdom, _listeners) = mounted(fake.clone());
        let html = dioxus_ssr::render(&vdom);

        assert!(fake.calls().is_empty(), "calls were {:?}", fake.calls());
        html
    });

    assert!(html.contains("Search…"), "html was {html}");
    assert!(
        html.contains("Type something to search local data."),
        "html was {html}"
    );
    assert!(html.contains("Courses, tasks, resources and announcements."));
}

#[test]
fn submitting_trims_the_query_and_fetches_the_hits() {
    let fake = FakeTransport::ok(search_payload());

    let html = common::with_locale(Language::En, || {
        let (mut vdom, listeners) = mounted(fake.clone());
        let input = listeners.input.expect("an input listener");
        let submit = listeners.submit.expect("a submit listener");

        let runtime = vdom.runtime();
        runtime.handle_event("input", form_event("  minería  "), input);
        runtime.handle_event("submit", form_event(""), submit);
        vdom.render_immediate(&mut NoOpMutations);
        vdom.render_immediate(&mut NoOpMutations);

        dioxus_ssr::render(&vdom)
    });

    assert_eq!(fake.last_command(), "search");
    assert_eq!(fake.last_args(), json!({ "query": "minería" }));
    assert!(html.contains("Lab 1"), "html was {html}");
    assert!(html.contains(">Activity<"), "html was {html}");
}

#[test]
fn zero_results_show_the_submitted_query() {
    let fake = FakeTransport::ok(json!({
        "query": "lab",
        "total": 0,
        "items": [],
    }));

    let html = common::with_locale(Language::En, || {
        let (mut vdom, listeners) = mounted(fake);
        let input = listeners.input.expect("an input listener");
        let submit = listeners.submit.expect("a submit listener");

        let runtime = vdom.runtime();
        runtime.handle_event("input", form_event("lab"), input);
        runtime.handle_event("submit", form_event(""), submit);
        vdom.render_immediate(&mut NoOpMutations);
        vdom.render_immediate(&mut NoOpMutations);

        dioxus_ssr::render(&vdom)
    });

    assert!(html.contains("No results for “lab”."), "html was {html}");
}

#[test]
fn an_empty_submission_does_not_fetch() {
    let fake = FakeTransport::ok(search_payload());

    common::with_locale(Language::En, || {
        let (mut vdom, listeners) = mounted(fake.clone());
        let input = listeners.input.expect("an input listener");
        let submit = listeners.submit.expect("a submit listener");

        let runtime = vdom.runtime();
        runtime.handle_event("input", form_event("   "), input);
        runtime.handle_event("submit", form_event(""), submit);
        vdom.render_immediate(&mut NoOpMutations);

        assert!(fake.calls().is_empty(), "calls were {:?}", fake.calls());
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Type something to search local data."),
            "html was {html}"
        );
    });
}
