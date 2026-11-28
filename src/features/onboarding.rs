use std::cell::Cell;
use std::rc::Rc;

use contracts::{SessionState, Source};
use dioxus::prelude::*;

use crate::api::Api;
use crate::components::{Alert, AlertVariant, Button, ButtonVariant, LanguageSelect};
use crate::domain::source_label;
use crate::error::{ERROR_KEY, ErrorKind};
use crate::i18n::{Language, resolve_language, t};
use crate::icons;
use crate::state::{AppContext, spawn_future, use_app};

const STALL_MS: u32 = 90_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnboardingState {
    #[default]
    Welcome,
    Connecting,
    Partial,
    Connected,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Pending,
    Active,
    Done,
    Failed,
}

/// The three connection steps in display order; `LoginProgress` walks
/// [`STEP_ORDER`], not the `Source` declaration order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Steps {
    pub ellucian: StepStatus,
    pub moodle: StepStatus,
    pub banner: StepStatus,
}

impl Steps {
    pub fn get(&self, source: Source) -> StepStatus {
        match source {
            Source::Ellucian => self.ellucian,
            Source::Moodle => self.moodle,
            Source::Banner => self.banner,
        }
    }

    pub fn set(&mut self, source: Source, status: StepStatus) {
        match source {
            Source::Ellucian => self.ellucian = status,
            Source::Moodle => self.moodle = status,
            Source::Banner => self.banner = status,
        }
    }
}

pub const STEP_ORDER: [Source; 3] = [Source::Ellucian, Source::Moodle, Source::Banner];

fn status_key(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "onboarding.steps.pending",
        StepStatus::Active => "onboarding.steps.active",
        StepStatus::Done => "onboarding.steps.done",
        StepStatus::Failed => "onboarding.steps.failed",
    }
}

fn status_name(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Active => "active",
        StepStatus::Done => "done",
        StepStatus::Failed => "failed",
    }
}

#[component]
pub fn LoginProgress(steps: Steps, #[props(default)] stalled_source: Option<Source>) -> Element {
    let aria = t("onboarding.steps.aria");

    rsx! {
        ol { class: "flex flex-col gap-3", aria_label: aria,
            for source in STEP_ORDER {
                LoginStep {
                    key: "{source:?}",
                    source,
                    status: steps.get(source),
                    stalled: stalled_source == Some(source),
                }
            }
        }
    }
}

#[component]
fn LoginStep(source: Source, status: StepStatus, stalled: bool) -> Element {
    let state = if stalled {
        "stalled"
    } else {
        status_name(status)
    };
    let label = if stalled {
        t("onboarding.steps.stalled")
    } else {
        t(status_key(status))
    };

    rsx! {
        li { class: "flex items-center gap-3 text-sm", "data-status": state,
            span { class: "flex size-5 items-center justify-center",
                if stalled || status == StepStatus::Failed {
                    {icons::x("size-4 text-destructive")}
                } else if status == StepStatus::Done {
                    {icons::check("size-4 text-emerald-600")}
                } else if status == StepStatus::Active {
                    {icons::loader("size-4 animate-spin text-muted-foreground")}
                } else {
                    {icons::circle("size-3 text-muted-foreground/50")}
                }
            }
            span { class: "min-w-24 font-medium", "{source_label(source)}" }
            span { class: "text-muted-foreground", "{label}" }
        }
    }
}

#[component]
pub fn OnboardingView(
    state: OnboardingState,
    steps: Steps,
    #[props(default)] failed_source: Option<Source>,
    #[props(default)] stalled_source: Option<Source>,
    #[props(default)] error: Option<ErrorKind>,
    on_connect: EventHandler<()>,
    on_retry: EventHandler<Source>,
    on_continue: EventHandler<()>,
) -> Element {
    let app = try_consume_context::<AppContext>();
    let language = match &app {
        Some(app) => (app.language)(),
        None => resolve_language(&rust_i18n::locale()),
    };
    let selector = language_control(language, app);

    if state == OnboardingState::Welcome {
        return rsx! {
            main { class: "flex min-h-svh items-center justify-center bg-background p-6",
                div { class: "flex w-full max-w-lg flex-col gap-6",
                    header { class: "flex items-start justify-between gap-3",
                        div { class: "flex flex-col gap-2",
                            h1 { class: "text-3xl font-semibold tracking-tight text-primary",
                                "rust-app"
                            }
                            p { class: "text-sm text-muted-foreground",
                                "{t(\"onboarding.tagline\")}"
                            }
                        }
                        {selector}
                    }
                    section { class: "flex flex-col gap-3 rounded-xl border border-border p-5",
                        div { class: "flex items-center gap-2 font-medium",
                            {icons::shield_check("size-4 text-primary")}
                            "{t(\"onboarding.beforeStart\")}"
                        }
                        ul { class: "flex list-disc flex-col gap-1 pl-5 text-sm text-muted-foreground",
                            li { "{t(\"onboarding.privacy\")}" }
                            li { "{t(\"onboarding.readOnly\")}" }
                            li { "{t(\"onboarding.password\")}" }
                        }
                    }
                    Button {
                        class: "h-10 w-full text-base",
                        onclick: move |_| on_connect.call(()),
                        "{t(\"onboarding.connect\")}"
                    }
                }
            }
        };
    }

    let (title_key, body_key) = match state {
        OnboardingState::Connected => ("onboarding.connectedTitle", "onboarding.connectedBody"),
        OnboardingState::Partial => ("onboarding.partialTitle", "onboarding.partialBody"),
        _ => ("onboarding.connectingTitle", "onboarding.connectingBody"),
    };
    let stalled_title = stalled_source
        .map(|source| t("onboarding.stalledTitle").replace("%{source}", source_label(source)));
    let failed_title = failed_source
        .map(|source| t("onboarding.failedTitle").replace("%{source}", source_label(source)));
    // While the loop is connecting, only a stalled step may be retried: a
    // failed source earlier in the list waits for the loop to finish, so two
    // logins never race for the shared auth window.
    let retry_target = match (state, stalled_source) {
        (OnboardingState::Connecting, Some(stalled)) => Some(stalled),
        (OnboardingState::Connecting, None) => None,
        _ => failed_source.or(stalled_source),
    };
    let can_continue = matches!(state, OnboardingState::Partial | OnboardingState::Connected);

    rsx! {
        main { class: "flex min-h-svh items-center justify-center bg-background p-6",
            div { class: "flex w-full max-w-lg flex-col gap-6",
                div { class: "flex items-start justify-between gap-3",
                    header { class: "flex flex-col gap-1",
                        h1 { class: "text-2xl font-semibold tracking-tight", "{t(title_key)}" }
                        p { class: "text-sm text-muted-foreground", "{t(body_key)}" }
                    }
                    {selector}
                }
                LoginProgress { steps, stalled_source }
                if let Some(title) = stalled_title {
                    Alert { variant: AlertVariant::Destructive,
                        div { class: "font-medium", "{title}" }
                        div { class: "text-muted-foreground", "{t(\"onboarding.stalledBody\")}" }
                    }
                }
                if let Some(title) = failed_title {
                    Alert { variant: AlertVariant::Destructive,
                        div { class: "font-medium", "{title}" }
                        div { class: "text-muted-foreground", "{t(\"onboarding.failedBody\")}" }
                    }
                }
                if let Some(kind) = error {
                    Alert { variant: AlertVariant::Destructive,
                        div { class: "font-medium", "{t(\"onboarding.unexpectedError\")}" }
                        div { class: "text-muted-foreground", "{t(ERROR_KEY[kind as usize])}" }
                    }
                }
                div { class: "flex flex-wrap gap-2",
                    if let Some(source) = retry_target {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| on_retry.call(source),
                            "{t(\"common.retry\")}"
                        }
                    }
                    if can_continue {
                        Button {
                            onclick: move |_| on_continue.call(()),
                            "{t(\"common.continue\")}"
                        }
                    }
                }
            }
        }
    }
}

fn language_control(language: Language, app: Option<AppContext>) -> Element {
    rsx! {
        LanguageSelect {
            class: "rounded-md border border-border bg-background px-2 py-1 text-sm",
            aria_label: t("onboarding.language"),
            value: language,
            on_change: move |next: Language| {
                if let Some(app) = &app {
                    app.set_language(next);
                    let api = app.api.clone();
                    spawn_future(async move {
                        let _ = api.set_language(next.code()).await;
                    });
                } else {
                    crate::i18n::set_language(next);
                }
            },
        }
    }
}

/// Attempt tokens, one slot per source in [`STEP_ORDER`]. A retry bumps only
/// its own source, so it cannot abort the other sources still in the loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Attempts([u64; 3]);

impl Attempts {
    fn begin(&mut self, source: Source) -> u64 {
        let slot = &mut self.0[source_index(source)];
        *slot = slot.wrapping_add(1);
        *slot
    }

    fn is_current(&self, source: Source, attempt: u64) -> bool {
        self.0[source_index(source)] == attempt
    }
}

const fn source_index(source: Source) -> usize {
    match source {
        Source::Ellucian => 0,
        Source::Moodle => 1,
        Source::Banner => 2,
    }
}

/// The terminal state once no step is in flight: `Connected` needs all three
/// steps done (and a connected session after a retry); a terminal failure is
/// `Partial`. `None` keeps the view in `Connecting`, so a retry still running
/// or a source not attempted yet can never be shown as a finished flow.
fn settled_state(steps: &Steps, session_connected: bool) -> Option<OnboardingState> {
    let mut all_done = true;

    for source in STEP_ORDER {
        match steps.get(source) {
            StepStatus::Done => {}
            StepStatus::Pending | StepStatus::Active => return None,
            StepStatus::Failed => all_done = false,
        }
    }

    Some(if all_done && session_connected {
        OnboardingState::Connected
    } else {
        OnboardingState::Partial
    })
}

#[derive(Clone, Copy)]
struct Flow {
    state: Signal<OnboardingState>,
    steps: Signal<Steps>,
    failed_source: Signal<Option<Source>>,
    stalled_source: Signal<Option<Source>>,
    error: Signal<Option<ErrorKind>>,
    session: Signal<Option<SessionState>>,
    attempts: Signal<Attempts>,
}

impl Flow {
    fn begin(&mut self, source: Source) -> u64 {
        let mut attempts = *self.attempts.peek();
        let attempt = attempts.begin(source);
        self.attempts.set(attempts);
        attempt
    }

    fn is_current(&self, source: Source, attempt: u64) -> bool {
        self.attempts.peek().is_current(source, attempt)
    }
}

#[component]
pub fn Onboarding(#[props(default)] on_continue: EventHandler<()>) -> Element {
    let app = use_app();
    let state = use_signal(|| OnboardingState::Welcome);
    let steps = use_signal(Steps::default);
    let failed_source = use_signal(|| None::<Source>);
    let stalled_source = use_signal(|| None::<Source>);
    let error = use_signal(|| None::<ErrorKind>);
    let session = app.session;
    let attempts = use_signal(Attempts::default);

    let flow = Flow {
        state,
        steps,
        failed_source,
        stalled_source,
        error,
        session,
        attempts,
    };
    let api = app.api.clone();
    let connect_api = api.clone();
    let retry_api = api;

    rsx! {
        OnboardingView {
            state: state(),
            steps: steps(),
            failed_source: failed_source(),
            stalled_source: stalled_source(),
            error: error(),
            on_connect: move |_| connect_all(connect_api.clone(), flow),
            on_retry: move |source| retry(retry_api.clone(), source, flow),
            on_continue: move |_| on_continue.call(()),
        }
    }
}

fn connect_all(api: Api, mut flow: Flow) {
    flow.state.set(OnboardingState::Connecting);
    flow.steps.set(Steps::default());
    flow.failed_source.set(None);
    flow.stalled_source.set(None);
    flow.error.set(None);

    spawn_future(async move {
        for source in STEP_ORDER {
            // A retry of this source supersedes the attempt, not the flow:
            // keep walking so the remaining sources are still attempted.
            let _ = run_source(&api, source, flow).await;
        }

        let steps = *flow.steps.peek();
        let session_connected = flow
            .session
            .peek()
            .as_ref()
            .is_some_and(|state| *state == SessionState::Connected);
        if let Some(next) = settled_state(&steps, session_connected) {
            flow.state.set(next);
        }
    });
}

fn retry(api: Api, source: Source, mut flow: Flow) {
    // The view hides this action while the loop is connecting, except for a
    // stalled step. Guard the handler too so a stray call cannot start a second
    // login for a source the loop has not settled.
    if *flow.state.peek() == OnboardingState::Connecting && flow.stalled_source.peek().is_none() {
        return;
    }
    flow.state.set(OnboardingState::Connecting);

    spawn_future(async move {
        let Some(result) = run_source(&api, source, flow).await else {
            return;
        };
        let session = api.refresh_session().await.ok();
        if !flow.is_current(source, result.token) {
            return;
        }

        if let Some(next) = &session {
            flow.session.set(Some(next.clone()));
        }

        if result.ok {
            let remaining = STEP_ORDER
                .into_iter()
                .find(|candidate| flow.steps.peek().get(*candidate) == StepStatus::Failed);
            flow.failed_source.set(remaining);
            flow.stalled_source.set(None);
            flow.error.set(None);
        }

        let steps = *flow.steps.peek();
        let session_connected = matches!(session, Some(SessionState::Connected));
        if let Some(next) = settled_state(&steps, session_connected) {
            flow.state.set(next);
        }
    });
}

/// Result of one current attempt. `None` from [`run_source`] means a retry for
/// the same source answered first; the late result must not touch the flow.
struct AttemptResult {
    token: u64,
    ok: bool,
}

/// One login attempt: active, connect, then done or failed. The stall timer is
/// disarmed when the attempt finishes so a late timer cannot touch a later
/// retry of the same source.
async fn run_source(api: &Api, source: Source, mut flow: Flow) -> Option<AttemptResult> {
    let token = flow.begin(source);
    flow.steps.write().set(source, StepStatus::Active);
    flow.stalled_source.set(None);

    let watch = watch_stall(source, token, flow);
    let result = api.connect(Some(source)).await;
    watch.disarm();

    if !flow.is_current(source, token) {
        return None;
    }
    flow.stalled_source.set(None);

    let ok = match result {
        Ok(next) => {
            flow.steps.write().set(source, StepStatus::Done);
            flow.session.set(Some(next));
            true
        }
        Err(error) => {
            flow.steps.write().set(source, StepStatus::Failed);
            flow.failed_source.set(Some(source));
            flow.error.set(Some(error.kind));
            false
        }
    };

    Some(AttemptResult { token, ok })
}

struct StallWatch(Rc<Cell<bool>>);

impl StallWatch {
    fn disarm(&self) {
        self.0.set(true);
    }
}

fn watch_stall(source: Source, attempt: u64, flow: Flow) -> StallWatch {
    let done = Rc::new(Cell::new(false));
    let finished = Rc::clone(&done);

    spawn_future(async move {
        gloo_timers::future::TimeoutFuture::new(STALL_MS).await;

        if !finished.get()
            && flow.is_current(source, attempt)
            && flow.steps.peek().get(source) == StepStatus::Active
        {
            let mut stalled_source = flow.stalled_source;
            stalled_source.set(Some(source));
        }
    });

    StallWatch(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps_with(source: Source, status: StepStatus) -> Steps {
        let mut steps = Steps::default();
        steps.set(source, status);
        steps
    }

    #[test]
    fn a_retry_only_invalidates_its_own_source() {
        let mut attempts = Attempts::default();
        let ellucian = attempts.begin(Source::Ellucian);
        let moodle = attempts.begin(Source::Moodle);

        let retry = attempts.begin(Source::Ellucian);

        assert!(!attempts.is_current(Source::Ellucian, ellucian));
        assert!(attempts.is_current(Source::Ellucian, retry));
        assert!(
            attempts.is_current(Source::Moodle, moodle),
            "the retry must not abort the source the loop is on"
        );
    }

    #[test]
    fn a_step_in_flight_keeps_the_flow_connecting() {
        let mut steps = Steps {
            ellucian: StepStatus::Done,
            moodle: StepStatus::Active,
            banner: StepStatus::Pending,
        };
        assert_eq!(settled_state(&steps, true), None);

        steps.set(Source::Moodle, StepStatus::Done);
        steps.set(Source::Banner, StepStatus::Failed);
        assert_eq!(settled_state(&steps, true), Some(OnboardingState::Partial));
    }

    #[test]
    fn connected_needs_all_three_steps_done() {
        let all_done = Steps {
            ellucian: StepStatus::Done,
            moodle: StepStatus::Done,
            banner: StepStatus::Done,
        };
        assert_eq!(
            settled_state(&all_done, true),
            Some(OnboardingState::Connected)
        );
        assert_eq!(
            settled_state(&all_done, false),
            Some(OnboardingState::Partial),
            "a retry also needs a connected session"
        );

        let one_failed_rest_pending = steps_with(Source::Banner, StepStatus::Failed);
        assert_eq!(
            settled_state(&one_failed_rest_pending, true),
            None,
            "a failed row with sources still pending is not terminal"
        );

        let mut one_failed_done_rest = all_done;
        one_failed_done_rest.set(Source::Banner, StepStatus::Failed);
        assert_eq!(
            settled_state(&one_failed_done_rest, true),
            Some(OnboardingState::Partial),
            "a failed row can never produce Connected"
        );
    }
}
