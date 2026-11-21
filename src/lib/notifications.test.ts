import {
    isPermissionGranted,
    requestPermission,
} from '@tauri-apps/plugin-notification';
import { beforeEach, expect, test, vi } from 'vitest';

import { ensureNotificationPermission } from './notifications';

vi.mock('@tauri-apps/plugin-notification', () => ({
    isPermissionGranted: vi.fn(),
    requestPermission: vi.fn(),
}));

beforeEach(() => {
    vi.clearAllMocks();
});

test('uses the existing permission', async () => {
    vi.mocked(isPermissionGranted).mockResolvedValue(true);

    await expect(ensureNotificationPermission()).resolves.toBe(true);
    expect(requestPermission).not.toHaveBeenCalled();
});

test('requests permission when needed', async () => {
    vi.mocked(isPermissionGranted).mockResolvedValue(false);
    vi.mocked(requestPermission).mockResolvedValue('granted');

    await expect(ensureNotificationPermission()).resolves.toBe(true);
    expect(requestPermission).toHaveBeenCalled();
});

test('returns false when the request is denied', async () => {
    vi.mocked(isPermissionGranted).mockResolvedValue(false);
    vi.mocked(requestPermission).mockResolvedValue('denied');

    await expect(ensureNotificationPermission()).resolves.toBe(false);
});
