import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/badge';

export type SyncState = 'idle' | 'running' | 'ok' | 'partial' | 'failed';

const SYNC_LABEL = {
    idle: 'shell.notSynced',
    running: 'shell.syncStatus.running',
    ok: 'shell.syncStatus.ok',
    partial: 'shell.syncStatus.partial',
    failed: 'shell.syncStatus.failed',
} as const;

export function SyncBadge({ state }: { state: SyncState }) {
    const { t } = useTranslation();

    return (
        <Badge variant={state === 'running' ? 'secondary' : 'outline'}>
            {t(SYNC_LABEL[state])}
        </Badge>
    );
}
