import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { appError, ERROR_KEY } from '@/lib/errors';

interface ScreenProps {
    title: string;
    subtitle?: string;
    actions?: ReactNode;
    children: ReactNode;
}

export function Screen({ title, subtitle, actions, children }: ScreenProps) {
    return (
        <section className="flex flex-col gap-6">
            <header className="flex flex-wrap items-start justify-between gap-3">
                <div className="flex flex-col gap-1">
                    <h2 className="text-xl font-semibold tracking-tight">
                        {title}
                    </h2>
                    {subtitle ? (
                        <p className="text-sm text-muted-foreground">
                            {subtitle}
                        </p>
                    ) : null}
                </div>
                {actions}
            </header>
            {children}
        </section>
    );
}

export function Loading() {
    const { t } = useTranslation();
    return (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
    );
}

export function ErrorNote({ error }: { error: unknown }) {
    const { t } = useTranslation();
    const parsed = appError(error);

    console.error(error);

    return (
        <p className="rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive">
            {t(ERROR_KEY[parsed.kind])}
        </p>
    );
}

export function EmptyNote({ children }: { children: ReactNode }) {
    return (
        <p className="rounded-lg border border-dashed border-border p-6 text-sm text-muted-foreground">
            {children}
        </p>
    );
}
