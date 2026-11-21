import type { ReactNode } from 'react';

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
    return <p className="text-sm text-muted-foreground">Cargando…</p>;
}

export function ErrorNote({ error }: { error: unknown }) {
    return (
        <p className="rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive">
            {String(error)}
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
