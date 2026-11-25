import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import {
    createMemoryHistory,
    createRootRoute,
    createRoute,
    createRouter,
    Outlet,
    RouterProvider,
} from '@tanstack/react-router';
import { fireEvent, render, screen } from '@testing-library/react';
import { vi } from 'vitest';

import { parseCourseSearch } from '@/app/router';
import { api, type CourseDetailDto } from '@/lib/api';

import { CourseDetailScreen } from './CourseDetail';

vi.mock('@/lib/api', () => ({
    api: {
        getCourse: vi.fn(),
    },
}));

const detail = {
    group: { id: 'g1', code: 'IGR-001', name: 'Mineria', sections: [] },
    tasks: [{ id: 'a1', name: 'Laboratorio 1' }],
    resources: [],
    grades: [],
    schedule: [],
} as unknown as CourseDetailDto;

it('honours the focus param and returns to the previous screen', async () => {
    vi.mocked(api.getCourse).mockResolvedValue(detail);
    Element.prototype.scrollIntoView = vi.fn();

    const root = createRootRoute({ component: Outlet });
    const search = createRoute({
        getParentRoute: () => root,
        path: '/buscar',
        component: () => <div>search screen</div>,
    });
    const course = createRoute({
        getParentRoute: () => root,
        path: '/ramos/$groupId',
        validateSearch: parseCourseSearch,
        component: CourseDetailScreen,
    });
    const router = createRouter({
        routeTree: root.addChildren([search, course]),
        history: createMemoryHistory({
            initialEntries: ['/buscar', '/ramos/g1?focus=a1'],
            initialIndex: 1,
        }),
    });
    await router.load();
    window.scrollTo = () => {};

    render(
        <QueryClientProvider client={new QueryClient()}>
            <RouterProvider router={router} />
        </QueryClientProvider>,
    );

    const row = (await screen.findByText('Laboratorio 1')).closest('li');
    expect(row).toHaveAttribute('aria-current', 'true');

    fireEvent.click(screen.getByRole('button', { name: 'Back' }));
    expect(await screen.findByText('search screen')).toBeInTheDocument();
});
