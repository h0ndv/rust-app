use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Outline,
    Ghost,
    Destructive,
}

impl ButtonVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
            Self::Outline => "border border-border bg-background hover:bg-muted",
            Self::Ghost => "hover:bg-muted",
            Self::Destructive => "bg-destructive text-white hover:bg-destructive/90",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let classes = format!(
        "inline-flex h-9 items-center justify-center gap-2 rounded-md px-3 text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50 {} {class}",
        variant.class()
    );

    rsx! {
        button {
            r#type: "button",
            class: classes,
            disabled,
            onclick: move |event| onclick.call(event),
            {children}
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Info,
    Warning,
    Destructive,
}

impl AlertVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Info => "border-info/40 bg-info/5 text-info-foreground",
            Self::Warning => "border-warning/40 bg-warning/5 text-warning-foreground",
            Self::Destructive => "border-destructive/40 bg-destructive/5 text-destructive",
        }
    }
}

#[component]
pub fn Alert(
    #[props(default)] variant: AlertVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let classes = format!(
        "flex flex-col gap-1 rounded-md border p-3 text-sm {} {class}",
        variant.class()
    );

    rsx! {
        div { role: "alert", class: classes, {children} }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Destructive,
    Success,
    Warning,
}

impl BadgeVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Default => "border-transparent bg-primary text-primary-foreground",
            Self::Secondary => "border-transparent bg-secondary text-secondary-foreground",
            Self::Outline => "border-border text-foreground",
            Self::Destructive => "border-transparent bg-destructive text-white",
            Self::Success => "border-transparent bg-success/15 text-success-foreground",
            Self::Warning => "border-transparent bg-warning/15 text-warning-foreground",
        }
    }
}

#[component]
pub fn Badge(
    #[props(default)] variant: BadgeVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let classes = format!(
        "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium {} {class}",
        variant.class()
    );

    rsx! {
        span { class: classes, {children} }
    }
}

#[component]
pub fn Progress(value: u8, #[props(default)] class: String) -> Element {
    let value = value.min(100);
    let classes = format!("h-2 w-full overflow-hidden rounded-full bg-muted {class}");

    rsx! {
        div {
            role: "progressbar",
            aria_valuemin: "0",
            aria_valuemax: "100",
            aria_valuenow: "{value}",
            class: classes,
            div {
                class: "h-full bg-primary transition-[width] duration-300",
                style: "width: {value}%",
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[component]
pub fn Separator(
    #[props(default)] orientation: Orientation,
    #[props(default)] class: String,
) -> Element {
    let vertical = orientation == Orientation::Vertical;
    let classes = if vertical {
        format!("w-px shrink-0 bg-border {class}")
    } else {
        format!("h-px w-full bg-border {class}")
    };

    rsx! {
        div {
            role: "separator",
            aria_orientation: if vertical { "vertical" } else { "horizontal" },
            class: classes,
        }
    }
}

#[component]
pub fn Spinner(#[props(default)] class: String) -> Element {
    let classes = format!(
        "inline-block size-4 animate-spin rounded-full border-2 border-current border-t-transparent {class}"
    );

    rsx! {
        span { class: classes, aria_hidden: "true" }
    }
}
