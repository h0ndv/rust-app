import {
    isPermissionGranted,
    sendNotification,
} from '@tauri-apps/plugin-notification';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, expect, test, vi } from 'vitest';

import { api, type DashboardDto, type SyncRun } from '@/lib/api';
import { onSyncFinished } from '@/lib/events';

import { useDueNotifications } from './use-notifications';

vi.mock('@/lib/api', () => ({
    api: {
        getSettings: vi.fn(),
        getDashboard: vi.fn(),
    },
}));

vi.mock('@/lib/events', () => ({
    onSyncFinished: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-notification', () => ({
    isPermissionGranted: vi.fn(),
    requestPermission: vi.fn(),
    sendNotification: vi.fn(),
}));

const settings = {
    sync_interval_minutes: 30,
    downloads_dir: '',
    theme: 'system',
    notifications: true,
    language: 'en',
};

const run: SyncRun = {
    id: 'run-1',
    status: 'ok',
    requests: 100,
    started_at: '2026-11-21T10:00:00Z',
    finished_at: '2026-11-21T10:01:00Z',
};

const dashboard: DashboardDto = {
    overdue: [
        {
            id: 'assign:1',
            course_id: 'moodle:1',
            course_name: 'Course',
            title: 'Task',
            due_at: null,
            status: 'overdue',
            submitted: false,
            grade: null,
            url: null,
            is_new: false,
        },
    ],
    due_soon: [],
    next_classes: [],
    new_resources: [],
};

let handler: ((run: SyncRun) => void) | undefined;
const unlisten = vi.fn();

beforeEach(() => {
    vi.clearAllMocks();
    handler = undefined;

    vi.mocked(api.getSettings).mockResolvedValue(settings);
    vi.mocked(api.getDashboard).mockResolvedValue(dashboard);
    vi.mocked(isPermissionGranted).mockResolvedValue(true);
    vi.mocked(onSyncFinished).mockImplementation(async (next) => {
        handler = next;
        return unlisten;
    });
});

test('sends a localized notification when there are due tasks', async () => {
    renderHook(() => useDueNotifications());

    await waitFor(() => expect(handler).toBeDefined());
    handler?.(run);

    await waitFor(() => {
        expect(sendNotification).toHaveBeenCalledWith({
            title: 'Pending work',
            body: '1 overdue task',
        });
    });
});

test('does not notify when the setting is disabled', async () => {
    vi.mocked(api.getSettings).mockResolvedValue({
        ...settings,
        notifications: false,
    });

    renderHook(() => useDueNotifications());

    await waitFor(() => expect(handler).toBeDefined());
    handler?.(run);

    await waitFor(() => {
        expect(api.getSettings).toHaveBeenCalled();
    });
    expect(api.getDashboard).not.toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
});

test('does not notify when permission is denied', async () => {
    vi.mocked(isPermissionGranted).mockResolvedValue(false);

    renderHook(() => useDueNotifications());

    await waitFor(() => expect(handler).toBeDefined());
    handler?.(run);

    await waitFor(() => {
        expect(api.getDashboard).toHaveBeenCalled();
    });
    expect(sendNotification).not.toHaveBeenCalled();
});

test('does not notify when nothing is due', async () => {
    vi.mocked(api.getDashboard).mockResolvedValue({
        overdue: [],
        due_soon: [],
        next_classes: [],
        new_resources: [],
    });

    renderHook(() => useDueNotifications());

    await waitFor(() => expect(handler).toBeDefined());
    handler?.(run);

    await waitFor(() => {
        expect(api.getDashboard).toHaveBeenCalled();
    });
    expect(sendNotification).not.toHaveBeenCalled();
});
