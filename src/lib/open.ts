import { toastManager } from '@/components/ui/toast';
import i18n from '@/i18n';

import { api } from './api';
import { appError, ERROR_KEY } from './errors';

export async function openInApp(url: string): Promise<void> {
    try {
        await api.openInApp(url);
    } catch (error) {
        const { kind } = appError(error);
        toastManager.add({
            type: 'error',
            title: i18n.t(ERROR_KEY[kind]),
        });
    }
}
