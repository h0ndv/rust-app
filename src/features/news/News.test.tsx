import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { vi } from 'vitest';

import { api, type NewsItemDto } from '@/lib/api';
import { renderRoute } from '@/test/router';

import { News } from './News';

vi.mock('@/lib/api', () => ({
    api: {
        acknowledge: vi.fn(),
    },
}));

const announcement: NewsItemDto = {
    id: 'moodle:1:2',
    course: 'Course',
    group_id: 'g1',
    title: 'Announcement',
    body: 'Body',
    published_at: null,
    kind: 'announcement',
};

const activity: NewsItemDto = {
    ...announcement,
    id: 'a1',
    title: 'Lab 1',
    kind: 'activity',
};

const file: NewsItemDto = {
    ...announcement,
    id: 'f1',
    title: 'Guia',
    kind: 'file',
};

function renderNews(items: NewsItemDto[]) {
    const client = new QueryClient();

    return renderRoute(
        <QueryClientProvider client={client}>
            <News items={items} />
        </QueryClientProvider>,
    );
}

test('marks an item as read', async () => {
    vi.mocked(api.acknowledge).mockResolvedValue(undefined);

    await renderNews([announcement]);

    fireEvent.click(screen.getByRole('button', { name: /mark as read/i }));

    await waitFor(() => {
        expect(api.acknowledge).toHaveBeenCalledWith(
            'announcement',
            announcement.id,
        );
    });
});

test('links an activity to its course and focus', async () => {
    await renderNews([activity]);

    expect(screen.getByRole('link', { name: /open course/i })).toHaveAttribute(
        'href',
        '/ramos/g1?focus=a1',
    );
});

test('links a file to its course and focus', async () => {
    await renderNews([file]);

    expect(screen.getByRole('link', { name: /open course/i })).toHaveAttribute(
        'href',
        '/ramos/g1?focus=f1',
    );
});

test('links an announcement without a focus', async () => {
    await renderNews([announcement]);

    expect(screen.getByRole('link', { name: /open course/i })).toHaveAttribute(
        'href',
        '/ramos/g1',
    );
});
