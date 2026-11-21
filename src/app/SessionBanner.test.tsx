import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, expect, test, vi } from 'vitest';

import type { SessionState, Source } from '@/lib/api';

import { SessionBanner } from './SessionBanner';
import { useSession } from './use-session';

vi.mock('./use-session', () => ({
    useSession: vi.fn(),
}));

const connect = vi.fn(async (_source?: Source) => {});

function mockSession(state: SessionState | null, connecting = false) {
    vi.mocked(useSession).mockReturnValue({ state, connecting, connect });
}

beforeEach(() => {
    vi.clearAllMocks();
});

test('offers connect when there is no session', () => {
    mockSession({ state: 'disconnected' });

    render(<SessionBanner />);

    expect(screen.getByText('No session')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Connect' }));
    expect(connect).toHaveBeenCalledWith(undefined);
});

test('names the expired source', () => {
    mockSession({ state: 'expired', source: 'moodle' });

    render(<SessionBanner />);

    expect(screen.getByText('Session expired')).toBeInTheDocument();
    expect(screen.getByText(/Moodle needs a new session/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect' }));
    expect(connect).toHaveBeenCalledWith('moodle');
});

test('hides when connected', () => {
    mockSession({ state: 'connected' });

    const { container } = render(<SessionBanner />);

    expect(container).toBeEmptyDOMElement();
});
