import { useTranslation } from 'react-i18next';

import type { RecoveryNoticeDto } from '@/lib/api';

export function RecoveryBanner({
    notice,
    onDismiss,
}: {
    notice: RecoveryNoticeDto;
    onDismiss: () => void;
}) {
    const { t } = useTranslation();

    return (
        <div
            role="status"
            className="m-4 flex flex-wrap items-center gap-3 rounded-lg border border-destructive/40 bg-destructive/5 p-3 text-sm"
        >
            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <span className="font-medium">{t('recovery.title')}</span>
                <span className="text-xs text-muted-foreground">
                    {t('recovery.body', { path: notice.backup_path })}
                </span>
            </div>
            <button
                type="button"
                className="rounded-md border border-border px-3 py-1 text-sm"
                onClick={onDismiss}
            >
                {t('recovery.dismiss')}
            </button>
        </div>
    );
}
