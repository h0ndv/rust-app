import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { vi } from 'vitest';

import { api, type NewsItemDto } from '@/lib/api';

import { News } from './News';

vi.mock('@/lib/api', () => ({
    api: {
        acknowledge: vi.fn(),
    },
}));

const item: NewsItemDto = {
    id: 'moodle:1:2',
    course: 'Course',
    title: 'Announcement',
    body: 'Body',
    published_at: null,
    kind: 'announcement',
};

test('marks an item as read', async () => {
    vi.mocked(api.acknowledge).mockResolvedValue(undefined);
    const client = new QueryClient();

    render(
        <QueryClientProvider client={client}>
            <News items={[item]} />
        </QueryClientProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: /mark as read/i }));

    await waitFor(() => {
        expect(api.acknowledge).toHaveBeenCalledWith('announcement', item.id);
    });
});
