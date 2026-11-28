use dioxus::prelude::*;

use crate::components::AppShell;
use crate::features::course_detail::CourseDetailScreen;
use crate::features::courses::CoursesScreen;
use crate::features::grades::GradesScreen;
use crate::features::home::HomeScreen;
use crate::features::news::NewsScreen;
use crate::features::resources::ResourcesScreen;
use crate::features::schedule::ScheduleScreen;
use crate::features::search::SearchScreen;
use crate::features::settings::SettingsScreen;
use crate::features::tasks::TasksScreen;

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppShell)]
    #[route("/", HomeScreen)]
    Home {},
    #[route("/ramos", CoursesScreen)]
    Courses {},
    #[route("/ramos/:group_id?:focus", CourseDetailScreen)]
    CourseDetail { group_id: String, focus: Option<String> },
    #[route("/tareas", TasksScreen)]
    Tasks {},
    #[route("/recursos", ResourcesScreen)]
    Resources {},
    #[route("/notas", GradesScreen)]
    Grades {},
    #[route("/horario", ScheduleScreen)]
    Schedule {},
    #[route("/novedades", NewsScreen)]
    News {},
    #[route("/buscar", SearchScreen)]
    Search {},
    #[route("/ajustes", SettingsScreen)]
    Settings {},
}
