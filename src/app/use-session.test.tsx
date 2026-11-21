import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, expect, test, vi } from 'vitest';

import { api, type SessionState } from '@/lib/api';
import { onSessionChanged } from '@/lib/events';

import { useSession } from './use-session';

vi.mock('@/lib/api', () => ({
    api: {
        sessionState: vi.fn(),
        connect: vi.fn(),
    },
}));

vi.mock('@/lib/events', () => ({
    onSessionChanged: vi.fn(),
}));

let handler: ((state: SessionState) => void) | undefined;
const unlisten = vi.fn();

beforeEach(() => {
    vi.clearAllMocks();
    handler = undefined;

    vi.mocked(api.sessionState).mockResolvedValue({ state: 'disconnected' });
    vi.mocked(api.connect).mockResolvedValue({ state: 'connected' });
    vi.mocked(onSessionChanged).mockImplementation(async (next) => {
        handler = next;
        return unlisten;
    });
});

test('hydrates and follows session changes', async () => {
    const { result } = renderHook(() => useSession());

    await waitFor(() => {
        expect(result.current.state).toEqual({ state: 'disconnected' });
    });
    await waitFor(() => expect(handler).toBeDefined());

    act(() => handler?.({ state: 'expired', source: 'moodle' }));

    expect(result.current.state).toEqual({
        state: 'expired',
        source: 'moodle',
    });
});

test('connects a source and stores the result', async () => {
    const { result } = renderHook(() => useSession());

    await waitFor(() => expect(result.current.state).not.toBeNull());

    await act(async () => {
        await result.current.connect('moodle');
    });

    expect(api.connect).toHaveBeenCalledWith('moodle');
    expect(result.current.state).toEqual({ state: 'connected' });
});
