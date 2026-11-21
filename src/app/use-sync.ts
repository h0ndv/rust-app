import { useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useState } from 'react';

import type { Source, SyncRun, SyncStatus } from '@/lib/api';
import { api } from '@/lib/api';
import { onSyncFinished, onSyncProgress } from '@/lib/events';

import type { SyncState } from './SyncBadge';

export interface SyncView {
    state: SyncState;
    progress: number;
    source: Source | null;
    requests: number;
    errors: string[];
    lastRun: SyncRun | null;
}

const INITIAL: SyncView = {
    state: 'idle',
    progress: 0,
    source: null,
    requests: 0,
    errors: [],
    lastRun: null,
};

function fromStatus(status: SyncStatus): Partial<SyncView> {
    return {
        state: status.state,
        progress: status.progress,
        source: status.source,
        requests: status.requests,
        errors: status.errors,
    };
}

export function useSyncController() {
    const queryClient = useQueryClient();
    const [view, setView] = useState<SyncView>(INITIAL);

    useEffect(() => {
        let disposed = false;
        const unlisteners: (() => void)[] = [];

        async function register() {
            const stopProgress = await onSyncProgress((status) => {
                if (!disposed) {
                    setView((current) => ({
                        ...current,
                        ...fromStatus(status),
                    }));
                }
            });
            if (disposed) {
                stopProgress();
                return;
            }
            unlisteners.push(stopProgress);

            const stopFinished = await onSyncFinished((run) => {
                if (disposed) {
                    return;
                }
                setView((current) => ({
                    ...current,
                    state: run.status,
                    progress: 100,
                    lastRun: run,
                }));
                void queryClient.invalidateQueries();
            });
            if (disposed) {
                stopFinished();
                return;
            }
            unlisteners.push(stopFinished);
        }

        void register().catch(console.error);

        void api
            .syncStatus()
            .then((status) => {
                if (disposed) {
                    return;
                }
                setView((current) => {
                    if (current.state !== 'idle') {
                        return current;
                    }
                    if (status.state === 'ok' && status.progress === 0) {
                        return current;
                    }
                    return { ...current, ...fromStatus(status) };
                });
            })
            .catch(console.error);

        return () => {
            disposed = true;
            unlisteners.forEach((stop) => stop());
        };
    }, [queryClient]);

    const start = useCallback(async () => {
        try {
            const status = await api.startSync();
            setView((current) => ({ ...current, ...fromStatus(status) }));
        } catch (error) {
            console.error(error);
            setView((current) => ({ ...current, state: 'failed' }));
        }
    }, []);

    const cancel = useCallback(() => {
        void api.cancelSync();
    }, []);

    return { ...view, start, cancel };
}
