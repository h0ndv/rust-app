import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { SessionState, SyncRun, SyncStatus } from './api';

export function onSyncProgress(
    handler: (status: SyncStatus) => void,
): Promise<UnlistenFn> {
    return listen<SyncStatus>('sync://progress', (event) =>
        handler(event.payload),
    );
}

export function onSyncFinished(
    handler: (run: SyncRun) => void,
): Promise<UnlistenFn> {
    return listen<SyncRun>('sync://finished', (event) =>
        handler(event.payload),
    );
}

/** Reserved: the backend does not emit it yet, but the UI is ready. */
export function onSessionChanged(
    handler: (state: SessionState) => void,
): Promise<UnlistenFn> {
    return listen<SessionState>('session://changed', (event) =>
        handler(event.payload),
    );
}
