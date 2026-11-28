use dioxus::prelude::*;

use crate::i18n::t;
use crate::icons;
use crate::router::Route;

pub struct NavItem {
    pub route: Route,
    pub label_key: &'static str,
    pub icon: fn(&str) -> Element,
}

pub fn nav_items() -> [NavItem; 9] {
    [
        NavItem {
            route: Route::Home {},
            label_key: "nav.home",
            icon: icons::home,
        },
        NavItem {
            route: Route::Courses {},
            label_key: "nav.courses",
            icon: icons::book_open,
        },
        NavItem {
            route: Route::Tasks {},
            label_key: "nav.tasks",
            icon: icons::clipboard_list,
        },
        NavItem {
            route: Route::Resources {},
            label_key: "nav.resources",
            icon: icons::folder_open,
        },
        NavItem {
            route: Route::Grades {},
            label_key: "nav.grades",
            icon: icons::graduation_cap,
        },
        NavItem {
            route: Route::Schedule {},
            label_key: "nav.schedule",
            icon: icons::calendar_days,
        },
        NavItem {
            route: Route::News {},
            label_key: "nav.news",
            icon: icons::newspaper,
        },
        NavItem {
            route: Route::Search {},
            label_key: "nav.search",
            icon: icons::search,
        },
        NavItem {
            route: Route::Settings {},
            label_key: "nav.settings",
            icon: icons::settings,
        },
    ]
}

/// Course detail belongs to the courses entry, so the rail keeps that item lit
/// while a course is open.
fn is_active(current: &Route, target: &Route) -> bool {
    match (current, target) {
        (Route::CourseDetail { .. }, Route::Courses {}) => true,
        _ => std::mem::discriminant(current) == std::mem::discriminant(target),
    }
}

#[component]
pub fn Sidebar(collapsed: bool, current: Route, on_toggle: EventHandler<()>) -> Element {
    let width = if collapsed { "w-14" } else { "w-64" };

    rsx! {
        aside {
            class: "fixed inset-y-0 left-0 z-30 flex {width} flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground transition-[width] duration-200",
            div { class: "flex h-14 items-center gap-2 border-b border-sidebar-border px-3",
                span {
                    class: if collapsed {
                        "sr-only"
                    } else {
                        "truncate text-sm font-semibold tracking-tight text-primary"
                    },
                    "rust-app"
                }
                button {
                    r#type: "button",
                    class: "ml-auto inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                    aria_label: if collapsed { t("shell.expand") } else { t("shell.collapse") },
                    aria_expanded: if collapsed { "false" } else { "true" },
                    title: if collapsed { t("shell.expand") } else { t("shell.collapse") },
                    onclick: move |_| on_toggle.call(()),
                    if collapsed {
                        {icons::chevron_right("size-4")}
                    } else {
                        {icons::chevron_left("size-4")}
                    }
                }
            }
            nav {
                class: "flex flex-1 flex-col gap-1 overflow-y-auto p-2",
                aria_label: t("shell.navigation"),
                span {
                    class: if collapsed {
                        "sr-only"
                    } else {
                        "px-2 py-1 text-xs font-medium text-muted-foreground"
                    },
                    "{t(\"shell.navigation\")}"
                }
                for item in nav_items() {
                    Link {
                        key: "{item.label_key}",
                        class: if is_active(&current, &item.route) {
                            "flex h-8 items-center gap-2 rounded-md bg-sidebar-accent px-2 text-sm font-medium text-sidebar-accent-foreground"
                        } else {
                            "flex h-8 items-center gap-2 rounded-md px-2 text-sm font-medium text-sidebar-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
                        },
                        to: item.route.clone(),
                        aria_current: if is_active(&current, &item.route) { "page" } else { "false" },
                        aria_label: if collapsed { Some(t(item.label_key)) } else { None },
                        title: if collapsed { Some(t(item.label_key)) } else { None },
                        {(item.icon)("size-4")}
                        if !collapsed {
                            span { class: "truncate", "{t(item.label_key)}" }
                        }
                    }
                }
            }
            div { class: "border-t border-sidebar-border p-2",
                div {
                    class: if collapsed {
                        "sr-only"
                    } else {
                        "px-2 py-1 text-xs text-muted-foreground"
                    },
                    "{t(\"shell.account\")}"
                }
            }
        }
    }
}
