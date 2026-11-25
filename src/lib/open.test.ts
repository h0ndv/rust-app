import { beforeEach, describe, expect, it, vi } from 'vitest';

import { toastManager } from '@/components/ui/toast';

import { api } from './api';
import { openInApp } from './open';

vi.mock('./api', () => ({ api: { openInApp: vi.fn() } }));
vi.mock('@/components/ui/toast', () => ({
    toastManager: { add: vi.fn() },
}));

describe('openInApp', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('surfaces a rejected link instead of failing silently', async () => {
        vi.mocked(api.openInApp).mockRejectedValueOnce({
            kind: 'forbidden',
            message: 'Enlace no permitido',
        });

        await expect(openInApp('https://evil.com/')).resolves.toBeUndefined();

        expect(toastManager.add).toHaveBeenCalledWith({
            type: 'error',
            title: "This link points outside the app's trusted sites.",
        });
    });

    it('stays quiet when the link opens', async () => {
        vi.mocked(api.openInApp).mockResolvedValueOnce(undefined);

        await openInApp('https://aulasvirtuales.santotomas.cl/');

        expect(toastManager.add).not.toHaveBeenCalled();
    });
});
