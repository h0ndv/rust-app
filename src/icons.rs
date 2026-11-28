use dioxus::prelude::*;

fn icon(class: &str, children: Element) -> Element {
    rsx! {
        svg {
            class: "size-4 shrink-0 {class}",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            {children}
        }
    }
}

pub fn home(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
            polyline { points: "9 22 9 12 15 12 15 22" }
        },
    )
}

pub fn book_open(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M12 7v14" }
            path { d: "M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z" }
        },
    )
}

pub fn clipboard_list(class: &str) -> Element {
    icon(
        class,
        rsx! {
            rect { width: "8", height: "4", x: "8", y: "2", rx: "1", ry: "1" }
            path { d: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" }
            path { d: "M12 11h4" }
            path { d: "M12 16h4" }
            path { d: "M8 11h.01" }
            path { d: "M8 16h.01" }
        },
    )
}

pub fn folder_open(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2" }
        },
    )
}

pub fn graduation_cap(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M21.42 10.922a1 1 0 0 0-.019-1.838L12.83 5.18a2 2 0 0 0-1.66 0L2.6 9.08a1 1 0 0 0 0 1.832l8.57 3.908a2 2 0 0 0 1.66 0z" }
            path { d: "M22 10v6" }
            path { d: "M6 12.5V16a6 3 0 0 0 12 0v-3.5" }
        },
    )
}

pub fn calendar_days(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M8 2v4" }
            path { d: "M16 2v4" }
            rect { width: "18", height: "18", x: "3", y: "4", rx: "2" }
            path { d: "M3 10h18" }
            path { d: "M8 14h.01" }
            path { d: "M12 14h.01" }
            path { d: "M16 14h.01" }
            path { d: "M8 18h.01" }
            path { d: "M12 18h.01" }
            path { d: "M16 18h.01" }
        },
    )
}

pub fn newspaper(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2" }
            path { d: "M18 14h-8" }
            path { d: "M15 18h-5" }
            path { d: "M10 6h8v4h-8V6Z" }
        },
    )
}

pub fn search(class: &str) -> Element {
    icon(
        class,
        rsx! {
            circle { cx: "11", cy: "11", r: "8" }
            path { d: "m21 21-4.3-4.3" }
        },
    )
}

pub fn settings(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }
            circle { cx: "12", cy: "12", r: "3" }
        },
    )
}

pub fn check(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M20 6 9 17l-5-5" }
        },
    )
}

pub fn circle(class: &str) -> Element {
    icon(
        class,
        rsx! {
            circle { cx: "12", cy: "12", r: "10" }
        },
    )
}

pub fn loader(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
        },
    )
}

pub fn x(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        },
    )
}

pub fn shield_check(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" }
            path { d: "m9 12 2 2 4-4" }
        },
    )
}

pub fn chevron_left(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "m15 18-6-6 6-6" }
        },
    )
}

pub fn chevron_right(class: &str) -> Element {
    icon(
        class,
        rsx! {
            path { d: "m9 18 6-6-6-6" }
        },
    )
}
