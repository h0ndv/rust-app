#![cfg(not(target_arch = "wasm32"))]

mod common;

use contracts::Source;
use dioxus::prelude::*;
use rust_app_ui::error::ErrorKind;
use rust_app_ui::features::onboarding::{
    LoginProgress, OnboardingState, OnboardingView, STEP_ORDER, StepStatus, Steps,
};
use rust_app_ui::i18n::Language;

#[component]
fn OnboardingHarness(
    state: OnboardingState,
    steps: Steps,
    #[props(default)] failed_source: Option<Source>,
    #[props(default)] stalled_source: Option<Source>,
    #[props(default)] error: Option<ErrorKind>,
) -> Element {
    rsx! {
        OnboardingView {
            state,
            steps,
            failed_source,
            stalled_source,
            error,
            on_connect: |_| {},
            on_retry: |_| {},
            on_continue: |_| {},
        }
    }
}

fn render(
    state: OnboardingState,
    steps: Steps,
    failed_source: Option<Source>,
    stalled_source: Option<Source>,
    error: Option<ErrorKind>,
) -> String {
    dioxus_ssr::render_element(rsx! {
        OnboardingHarness { state, steps, failed_source, stalled_source, error }
    })
}

#[component]
fn LoginProgressHarness(steps: Steps, stalled_source: Option<Source>) -> Element {
    rsx! {
        LoginProgress { steps, stalled_source }
    }
}

#[test]
fn steps_start_pending_and_are_indexed_by_source() {
    let mut steps = Steps::default();
    for source in STEP_ORDER {
        assert_eq!(steps.get(source), StepStatus::Pending, "{source:?}");
    }

    steps.set(Source::Moodle, StepStatus::Active);
    steps.set(Source::Banner, StepStatus::Failed);

    assert_eq!(steps.get(Source::Ellucian), StepStatus::Pending);
    assert_eq!(steps.get(Source::Moodle), StepStatus::Active);
    assert_eq!(steps.get(Source::Banner), StepStatus::Failed);
    assert_eq!(
        STEP_ORDER,
        [Source::Ellucian, Source::Moodle, Source::Banner]
    );
}

#[test]
fn welcome_explains_privacy_connect_and_language() {
    let html = common::with_locale(Language::En, || {
        render(OnboardingState::Welcome, Steps::default(), None, None, None)
    });

    assert!(
        html.contains("Your courses, tasks, schedule and grades in one place."),
        "html was {html}"
    );
    assert!(html.contains("Before you start"), "html was {html}");
    assert!(
        html.contains("There is no central server and no telemetry."),
        "html was {html}"
    );
    assert!(
        html.contains("It never submits, uploads or changes anything."),
        "html was {html}"
    );
    assert!(
        html.contains("Your password and MFA are entered in the Microsoft window"),
        "html was {html}"
    );
    assert!(html.contains("Connect with Microsoft"), "html was {html}");
    assert!(html.contains("aria-label=\"Language\""), "html was {html}");
    assert!(html.contains("English"), "html was {html}");
    assert!(html.contains("Español"), "html was {html}");
    assert!(html.contains("value=\"en\""), "html was {html}");
}

#[test]
fn connecting_lists_the_three_steps_in_order() {
    let html = common::with_locale(Language::En, || {
        render(
            OnboardingState::Connecting,
            Steps::default(),
            None,
            None,
            None,
        )
    });

    let ellucian = html.find("Ellucian").expect("Ellucian step");
    let moodle = html.find("Moodle").expect("Moodle step");
    let banner = html.find("Banner").expect("Banner step");
    assert!(
        ellucian < moodle && moodle < banner,
        "steps out of order: {html}"
    );
    assert!(
        html.contains("aria-label=\"Connection steps\""),
        "html was {html}"
    );
    assert!(html.contains("Connecting to Microsoft"), "html was {html}");
    assert!(html.contains("Waiting"), "html was {html}");
    assert!(!html.contains("Continue"), "html was {html}");
}

#[test]
fn partial_with_a_failed_source_offers_retry_and_continue() {
    let steps = Steps {
        ellucian: StepStatus::Done,
        moodle: StepStatus::Done,
        banner: StepStatus::Failed,
    };

    let html = common::with_locale(Language::En, || {
        render(
            OnboardingState::Partial,
            steps,
            Some(Source::Banner),
            None,
            None,
        )
    });

    assert!(html.contains("Partly connected"), "html was {html}");
    assert!(html.contains("Banner could not connect"), "html was {html}");
    assert!(
        html.contains("Check the login window or retry the step."),
        "html was {html}"
    );
    assert!(html.contains("data-status=\"failed\""), "html was {html}");
    assert!(html.contains("Retry"), "html was {html}");
    assert!(html.contains("Continue"), "html was {html}");
}

#[test]
fn retry_is_hidden_while_the_connect_loop_is_running() {
    let steps = Steps {
        ellucian: StepStatus::Failed,
        moodle: StepStatus::Active,
        banner: StepStatus::Pending,
    };

    let html = common::with_locale(Language::En, || {
        render(
            OnboardingState::Connecting,
            steps,
            Some(Source::Ellucian),
            None,
            None,
        )
    });

    assert!(!html.contains("Retry"), "html was {html}");
    assert!(!html.contains("Continue"), "html was {html}");
    assert!(html.contains("data-status=\"active\""), "html was {html}");
}

#[test]
fn stalled_source_shows_the_alert_and_retry() {
    let steps = Steps {
        ellucian: StepStatus::Done,
        moodle: StepStatus::Active,
        banner: StepStatus::Pending,
    };

    let html = common::with_locale(Language::En, || {
        render(
            OnboardingState::Connecting,
            steps,
            None,
            Some(Source::Moodle),
            None,
        )
    });

    assert!(html.contains("Moodle is not responding"), "html was {html}");
    assert!(html.contains("No response"), "html was {html}");
    assert!(html.contains("data-status=\"stalled\""), "html was {html}");
    assert!(html.contains("Retry"), "html was {html}");
    assert!(!html.contains("Continue"), "html was {html}");
}

#[test]
fn connected_session_lets_the_student_in() {
    let steps = Steps {
        ellucian: StepStatus::Done,
        moodle: StepStatus::Done,
        banner: StepStatus::Done,
    };

    let html = common::with_locale(Language::En, || {
        render(OnboardingState::Connected, steps, None, None, None)
    });

    assert!(html.contains("All set"), "html was {html}");
    assert!(
        html.contains("All three sources responded."),
        "html was {html}"
    );
    assert!(html.contains("Continue"), "html was {html}");
    assert!(!html.contains("Retry"), "html was {html}");
    assert_eq!(html.matches("data-status=\"done\"").count(), 3);
}

#[test]
fn partial_without_a_failure_has_no_retry() {
    let html = common::with_locale(Language::En, || {
        render(OnboardingState::Partial, Steps::default(), None, None, None)
    });

    assert!(html.contains("Continue"), "html was {html}");
    assert!(!html.contains("Retry"), "html was {html}");
}

#[test]
fn command_error_is_localized_in_both_languages() {
    let english = common::with_locale(Language::En, || {
        render(
            OnboardingState::Partial,
            Steps::default(),
            None,
            None,
            Some(ErrorKind::Network),
        )
    });
    assert!(english.contains("Unexpected error"), "html was {english}");
    assert!(
        english.contains("Network error. Check your connection and try again."),
        "html was {english}"
    );

    let spanish = common::with_locale(Language::Es, || {
        render(
            OnboardingState::Partial,
            Steps::default(),
            None,
            None,
            Some(ErrorKind::Network),
        )
    });
    assert!(spanish.contains("Error de red"), "html was {spanish}");
}

#[test]
fn login_progress_marks_each_status() {
    let steps = Steps {
        ellucian: StepStatus::Done,
        moodle: StepStatus::Active,
        banner: StepStatus::Pending,
    };

    let html = common::with_locale(Language::En, || {
        dioxus_ssr::render_element(rsx! {
            LoginProgressHarness { steps, stalled_source: None }
        })
    });

    assert_eq!(html.matches("data-status=\"done\"").count(), 1);
    assert_eq!(html.matches("data-status=\"active\"").count(), 1);
    assert_eq!(html.matches("data-status=\"pending\"").count(), 1);
    assert!(html.contains("Done"), "html was {html}");
    assert!(html.contains("Waiting"), "html was {html}");
}
