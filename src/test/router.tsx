import {
    createMemoryHistory,
    createRootRoute,
    createRoute,
    createRouter,
    Outlet,
    RouterProvider,
} from '@tanstack/react-router';
import { render } from '@testing-library/react';
import type { ReactElement } from 'react';

export async function renderRoute(ui: ReactElement, path = '/') {
    const root = createRootRoute({ component: Outlet });
    const index = createRoute({
        getParentRoute: () => root,
        path: '/',
        component: () => ui,
    });
    const course = createRoute({
        getParentRoute: () => root,
        path: '/ramos/$groupId',
        component: () => null,
    });
    const router = createRouter({
        routeTree: root.addChildren([index, course]),
        history: createMemoryHistory({ initialEntries: [path] }),
    });

    await router.load();
    window.scrollTo = () => {};

    return render(<RouterProvider router={router} />);
}
