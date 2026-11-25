import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { LanguageSelect } from '@/components/language-select';
import i18n, { resolveLanguage } from '@/i18n';
import { api, type SettingsDto } from '@/lib/api';
import { ensureNotificationPermission } from '@/lib/notifications';
import { normalizeTheme, setTheme } from '@/lib/theme';

import { Loading, Screen } from '../common/Screen';

const THEMES = [
    { value: 'system', labelKey: 'settings.themes.system' as const },
    { value: 'light', labelKey: 'settings.themes.light' as const },
    { value: 'dark', labelKey: 'settings.themes.dark' as const },
];

type Status =
    | { kind: 'saved' }
    | { kind: 'save_failed' }
    | { kind: 'clear_failed' }
    | { kind: 'notifications_denied' }
    | { kind: 'cleared'; count: number }
    | { kind: 'downloads_cleared'; count: number }
    | { kind: 'exported'; path: string }
    | { kind: 'export_failed' }
    | { kind: 'clear_downloads_failed' }
    | null;

const STATUS_KEY = {
    saved: 'settings.saved',
    save_failed: 'settings.saveFailed',
    clear_failed: 'settings.clearFailed',
    notifications_denied: 'settings.notificationsDenied',
    export_failed: 'settings.exportFailed',
    clear_downloads_failed: 'settings.clearDownloadsFailed',
} as const;

export function SettingsScreen() {
    const { t } = useTranslation();
    const queryClient = useQueryClient();
    const { data } = useQuery({
        queryKey: ['settings'],
        queryFn: api.getSettings,
    });

    const [form, setForm] = useState<SettingsDto | null>(null);
    const [status, setStatus] = useState<Status>(null);

    useEffect(() => {
        if (data && form === null) {
            setForm({
                ...data,
                language: resolveLanguage(data.language),
                theme: normalizeTheme(data.theme),
            });
        }
    }, [data, form]);

    async function save(): Promise<void> {
        if (!form) {
            return;
        }
        try {
            await api.saveSettings(form);
            queryClient.setQueryData(['settings'], form);
            await i18n.changeLanguage(form.language);
            document.documentElement.lang = form.language;
            setTheme(form.theme);

            const allowed = form.notifications
                ? await ensureNotificationPermission().catch(() => false)
                : true;
            setStatus(
                allowed ? { kind: 'saved' } : { kind: 'notifications_denied' },
            );
        } catch (cause) {
            console.error(cause);
            setStatus({ kind: 'save_failed' });
        }
    }

    async function clear(): Promise<void> {
        if (!window.confirm(t('settings.clearConfirm'))) {
            return;
        }
        try {
            const deleted = await api.clearData();
            await queryClient.invalidateQueries();
            setStatus({ kind: 'cleared', count: deleted });
        } catch (cause) {
            console.error(cause);
            setStatus({ kind: 'clear_failed' });
        }
    }

    async function exportData(): Promise<void> {
        try {
            const path = await api.exportData();
            setStatus({ kind: 'exported', path });
        } catch (cause) {
            console.error(cause);
            setStatus({ kind: 'export_failed' });
        }
    }

    async function clearDownloads(): Promise<void> {
        if (!window.confirm(t('settings.clearDownloadsConfirm'))) {
            return;
        }
        try {
            const removed = await api.clearDownloads();
            await queryClient.invalidateQueries({ queryKey: ['resources'] });
            setStatus({ kind: 'downloads_cleared', count: removed });
        } catch (cause) {
            console.error(cause);
            setStatus({ kind: 'clear_downloads_failed' });
        }
    }

    if (!form) {
        return (
            <Screen title={t('settings.title')}>
                <Loading />
            </Screen>
        );
    }

    return (
        <Screen title={t('settings.title')} subtitle={t('settings.subtitle')}>
            <div className="flex max-w-md flex-col gap-4">
                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">
                        {t('settings.language')}
                    </span>
                    <LanguageSelect
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.language}
                        onChange={(language) => setForm({ ...form, language })}
                    />
                </label>

                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">
                        {t('settings.syncInterval')}
                    </span>
                    <input
                        type="number"
                        min={5}
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.sync_interval_minutes}
                        onChange={(event) =>
                            setForm({
                                ...form,
                                sync_interval_minutes: Number(
                                    event.target.value,
                                ),
                            })
                        }
                    />
                </label>

                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">
                        {t('settings.downloadsDir')}
                    </span>
                    <input
                        type="text"
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.downloads_dir}
                        onChange={(event) =>
                            setForm({
                                ...form,
                                downloads_dir: event.target.value,
                            })
                        }
                    />
                </label>

                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">{t('settings.theme')}</span>
                    <select
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.theme}
                        onChange={(event) =>
                            setForm({ ...form, theme: event.target.value })
                        }
                    >
                        {THEMES.map((theme) => (
                            <option key={theme.value} value={theme.value}>
                                {t(theme.labelKey)}
                            </option>
                        ))}
                    </select>
                </label>

                <label className="flex items-center gap-2 text-sm">
                    <input
                        type="checkbox"
                        checked={form.notifications}
                        onChange={(event) =>
                            setForm({
                                ...form,
                                notifications: event.target.checked,
                            })
                        }
                    />
                    <span className="font-medium">
                        {t('settings.notifications')}
                    </span>
                </label>

                <div className="flex flex-wrap gap-2">
                    <button
                        type="button"
                        className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground"
                        onClick={save}
                    >
                        {t('settings.save')}
                    </button>
                </div>

                <section className="mt-4 flex flex-col gap-2 rounded-lg border border-border p-4">
                    <h3 className="text-sm font-semibold">
                        {t('settings.exportTitle')}
                    </h3>
                    <p className="text-xs text-muted-foreground">
                        {t('settings.exportBody')}
                    </p>
                    <button
                        type="button"
                        className="self-start rounded-md border border-border px-3 py-1 text-sm"
                        onClick={() => void exportData()}
                    >
                        {t('settings.exportButton')}
                    </button>
                </section>

                <section className="mt-4 flex flex-col gap-2 rounded-lg border border-destructive/40 p-4">
                    <h3 className="text-sm font-semibold">
                        {t('settings.clearTitle')}
                    </h3>
                    <p className="text-xs text-muted-foreground">
                        {t('settings.clearBody')}
                    </p>
                    <button
                        type="button"
                        className="self-start rounded-md border border-destructive px-3 py-1 text-sm text-destructive"
                        onClick={clear}
                    >
                        {t('settings.clearButton')}
                    </button>
                    <button
                        type="button"
                        className="self-start rounded-md border border-destructive px-3 py-1 text-sm text-destructive"
                        onClick={() => void clearDownloads()}
                    >
                        {t('settings.clearDownloadsButton')}
                    </button>
                </section>

                {status ? (
                    <p className="text-sm text-muted-foreground">
                        {status.kind === 'cleared'
                            ? t('settings.cleared', { count: status.count })
                            : status.kind === 'downloads_cleared'
                              ? t('settings.downloadsCleared', {
                                    count: status.count,
                                })
                              : status.kind === 'exported'
                                ? t('settings.exported', { path: status.path })
                                : t(STATUS_KEY[status.kind])}
                    </p>
                ) : null}
            </div>
        </Screen>
    );
}
