import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, expect, test, vi } from 'vitest';

import { api, type SyncRun, type SyncStatus } from '@/lib/api';
import { onSyncFinished, onSyncProgress } from '@/lib/events';

import { useSyncController } from './use-sync';

vi.mock('@/lib/api', () => ({
    api: {
        startSync: vi.fn(),
        cancelSync: vi.fn(),
        syncStatus: vi.fn(),
    },
}));

vi.mock('@/lib/events', () => ({
    onSyncProgress: vi.fn(),
    onSyncFinished: vi.fn(),
}));

const idle: SyncStatus = {
    state: 'ok',
    source: null,
    progress: 0,
    requests: 0,
    errors: [],
};

const running: SyncStatus = {
    state: 'running',
    source: 'moodle',
    progress: 40,
    requests: 12,
    errors: [],
};

const finished: SyncRun = {
    id: 'run-1',
    status: 'ok',
    requests: 100,
    started_at: '2026-11-21T10:00:00Z',
    finished_at: '2026-11-21T10:01:00Z',
};

let progressHandler: ((status: SyncStatus) => void) | undefined;
let finishedHandler: ((run: SyncRun) => void) | undefined;
const unlisten = vi.fn();
let client: QueryClient;

beforeEach(() => {
    client = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });

    vi.mocked(api.syncStatus).mockResolvedValue(idle);
    vi.mocked(api.startSync).mockResolvedValue(running);
    vi.mocked(api.cancelSync).mockResolvedValue(undefined);

    vi.mocked(onSyncProgress).mockImplementation(async (handler) => {
        progressHandler = handler;
        return unlisten;
    });
    vi.mocked(onSyncFinished).mockImplementation(async (handler) => {
        finishedHandler = handler;
        return unlisten;
    });
});

function wrapper({ children }: { children: ReactNode }) {
    return (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
}

test('follows progress and refreshes data when the run finishes', async () => {
    const invalidate = vi.spyOn(client, 'invalidateQueries');
    const { result } = renderHook(() => useSyncController(), { wrapper });

    await waitFor(() => {
        expect(progressHandler).toBeDefined();
        expect(finishedHandler).toBeDefined();
    });
    expect(result.current.state).toBe('idle');

    act(() => progressHandler?.(running));
    expect(result.current.state).toBe('running');
    expect(result.current.progress).toBe(40);
    expect(result.current.source).toBe('moodle');

    act(() => finishedHandler?.(finished));
    expect(result.current.state).toBe('ok');
    expect(result.current.progress).toBe(100);
    expect(result.current.lastRun?.id).toBe('run-1');

    await waitFor(() => {
        expect(invalidate).toHaveBeenCalled();
    });
});

test('starts without waiting for the whole run and can cancel', async () => {
    const { result } = renderHook(() => useSyncController(), { wrapper });

    await waitFor(() => expect(progressHandler).toBeDefined());

    await act(async () => {
        await result.current.start();
    });
    expect(result.current.state).toBe('running');

    act(() => result.current.cancel());
    expect(api.cancelSync).toHaveBeenCalled();
});
