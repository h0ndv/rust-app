import {
    isPermissionGranted,
    sendNotification,
} from '@tauri-apps/plugin-notification';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { api } from '@/lib/api';
import { onSyncFinished } from '@/lib/events';

export function useDueNotifications() {
    const { t } = useTranslation();

    useEffect(() => {
        let disposed = false;
        let stop: (() => void) | undefined;

        const register = async () => {
            const unlisten = await onSyncFinished(async (run) => {
                if (disposed || run.status === 'failed') {
                    return;
                }

                try {
                    const settings = await api.getSettings();
                    if (!settings.notifications) {
                        return;
                    }

                    const dashboard = await api.getDashboard();
                    const overdue = dashboard.overdue.length;
                    const dueSoon = dashboard.due_soon.length;
                    if (overdue + dueSoon === 0) {
                        return;
                    }

                    if (!(await isPermissionGranted())) {
                        return;
                    }

                    const parts: string[] = [];
                    if (overdue > 0) {
                        parts.push(
                            t('notifications.overdue', { count: overdue }),
                        );
                    }
                    if (dueSoon > 0) {
                        parts.push(
                            t('notifications.dueSoon', { count: dueSoon }),
                        );
                    }

                    sendNotification({
                        title: t('notifications.title'),
                        body: parts.join(' · '),
                    });
                } catch (error) {
                    console.error(error);
                }
            });

            if (disposed) {
                unlisten();
                return;
            }
            stop = unlisten;
        };

        void register().catch(console.error);

        return () => {
            disposed = true;
            stop?.();
        };
    }, [t]);
}
