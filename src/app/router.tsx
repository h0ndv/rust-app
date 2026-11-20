import {
    createRootRoute,
    createRoute,
    createRouter,
    Outlet,
} from '@tanstack/react-router';

import { AppShell } from './AppShell';

function Placeholder({ title }: { title: string }) {
    return (
        <section className="flex flex-col gap-2">
            <h2 className="text-xl font-semibold tracking-tight">{title}</h2>
            <p className="text-sm text-muted-foreground">
                Esta sección se conecta al backend en la siguiente tarea.
            </p>
        </section>
    );
}

function Root() {
    return (
        <AppShell>
            <Outlet />
        </AppShell>
    );
}

const rootRoute = createRootRoute({ component: Root });

const routes = [
    { path: '/', title: 'Inicio' },
    { path: '/ramos', title: 'Ramos' },
    { path: '/tareas', title: 'Tareas' },
    { path: '/recursos', title: 'Recursos' },
    { path: '/notas', title: 'Notas' },
    { path: '/horario', title: 'Horario' },
    { path: '/novedades', title: 'Novedades' },
    { path: '/buscar', title: 'Buscador' },
    { path: '/ajustes', title: 'Ajustes' },
].map(({ path, title }) =>
    createRoute({
        getParentRoute: () => rootRoute,
        path,
        component: () => <Placeholder title={title} />,
    }),
);

const detailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/ramos/$groupId',
    component: () => <Placeholder title="Detalle del ramo" />,
});

export const routeTree = rootRoute.addChildren([...routes, detailRoute]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
    interface Register {
        router: typeof router;
    }
}
