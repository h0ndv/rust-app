import {
    createRootRoute,
    createRoute,
    createRouter,
    Outlet,
} from '@tanstack/react-router';

import { CourseDetailScreen } from '@/features/courses/CourseDetail';
import { CoursesScreen } from '@/features/courses/Courses';
import { GradesScreen } from '@/features/grades/Grades';
import { HomeScreen } from '@/features/home/Home';
import { NewsScreen } from '@/features/news/News';
import { ResourcesScreen } from '@/features/resources/Resources';
import { ScheduleScreen } from '@/features/schedule/Schedule';
import { SearchScreen } from '@/features/search/Search';
import { SettingsScreen } from '@/features/settings/Settings';
import { TasksScreen } from '@/features/tasks/Tasks';

import { AppShell } from './AppShell';

function Root() {
    return (
        <AppShell>
            <Outlet />
        </AppShell>
    );
}

const rootRoute = createRootRoute({ component: Root });

const homeRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: HomeScreen,
});

const coursesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/ramos',
    component: CoursesScreen,
});

const courseDetailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/ramos/$groupId',
    component: CourseDetailScreen,
});

const tasksRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/tareas',
    component: TasksScreen,
});

const resourcesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/recursos',
    component: ResourcesScreen,
});

const gradesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/notas',
    component: GradesScreen,
});

const scheduleRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/horario',
    component: ScheduleScreen,
});

const newsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/novedades',
    component: NewsScreen,
});

const searchRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/buscar',
    component: SearchScreen,
});

const settingsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/ajustes',
    component: SettingsScreen,
});

export const routeTree = rootRoute.addChildren([
    homeRoute,
    coursesRoute,
    courseDetailRoute,
    tasksRoute,
    resourcesRoute,
    gradesRoute,
    scheduleRoute,
    newsRoute,
    searchRoute,
    settingsRoute,
]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
    interface Register {
        router: typeof router;
    }
}
