import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { RecoveryBanner } from './app/RecoveryBanner';
import { router } from './app/router';
import {
    STEP_ORDER,
    type StepStatus,
} from './features/onboarding/LoginProgress';
import {
    Onboarding,
    type OnboardingState,
} from './features/onboarding/Onboarding';
import { api, type RecoveryNoticeDto, type Source } from './lib/api';
import { appError, type ErrorKind } from './lib/errors';

const STALL_MS = 90_000;

const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: 30_000, retry: false } },
});

const INITIAL_STEPS: Record<Source, StepStatus> = {
    ellucian: 'pending',
    moodle: 'pending',
    banner: 'pending',
};

type Mode = 'loading' | 'onboarding' | 'app';

function App() {
    const { t } = useTranslation();
    const [mode, setMode] = useState<Mode>('loading');
    const [state, setState] = useState<OnboardingState>('welcome');
    const [steps, setSteps] =
        useState<Record<Source, StepStatus>>(INITIAL_STEPS);
    const [failedSource, setFailedSource] = useState<Source>();
    const [stalledSource, setStalledSource] = useState<Source>();
    const [error, setError] = useState<ErrorKind>();
    const [recovery, setRecovery] = useState<RecoveryNoticeDto | null>(null);

    useEffect(() => {
        let alive = true;

        void (async () => {
            try {
                const [hasData, session, notice] = await Promise.all([
                    api.hasLocalData().catch(() => false),
                    api
                        .sessionState()
                        .catch(() => ({ state: 'disconnected' }) as const),
                    api.recoveryNotice().catch(() => null),
                ]);
                if (!alive) {
                    return;
                }
                setRecovery(notice);
                setMode(
                    hasData || session.state !== 'disconnected'
                        ? 'app'
                        : 'onboarding',
                );
            } catch (cause) {
                console.error(cause);
                if (alive) {
                    setMode('onboarding');
                }
            }
        })();

        return () => {
            alive = false;
        };
    }, []);

    async function runStep(source: Source): Promise<boolean> {
        setSteps((current) => ({ ...current, [source]: 'active' }));
        setStalledSource(undefined);

        const timer = window.setTimeout(
            () => setStalledSource(source),
            STALL_MS,
        );

        try {
            await api.connect(source);
            window.clearTimeout(timer);
            setSteps((current) => ({ ...current, [source]: 'done' }));
            return true;
        } catch (cause) {
            window.clearTimeout(timer);
            setSteps((current) => ({ ...current, [source]: 'failed' }));
            setFailedSource(source);
            console.error(cause);
            setError(appError(cause).kind);
            return false;
        }
    }

    async function connectAll(): Promise<void> {
        setState('connecting');
        setFailedSource(undefined);
        setStalledSource(undefined);
        setError(undefined);

        let failed = false;
        for (const source of STEP_ORDER) {
            const ok = await runStep(source);
            failed = failed || !ok;
        }

        setState(failed ? 'partial' : 'connected');
    }

    async function retry(source: Source): Promise<void> {
        setState('connecting');
        const ok = await runStep(source);

        if (ok) {
            const session = await api.refreshSession().catch(() => null);
            setFailedSource(undefined);
            setError(undefined);
            setState(session?.state === 'connected' ? 'connected' : 'partial');
        } else {
            setState('partial');
        }
    }

    const banner = recovery ? (
        <RecoveryBanner notice={recovery} onDismiss={() => setRecovery(null)} />
    ) : null;

    if (mode === 'loading') {
        return (
            <>
                {banner}
                <main className="flex min-h-svh items-center justify-center bg-background p-6">
                    <p className="text-sm text-muted-foreground">
                        {t('common.loading')}
                    </p>
                </main>
            </>
        );
    }

    if (mode === 'onboarding') {
        return (
            <>
                {banner}
                <Onboarding
                    state={state}
                    steps={steps}
                    failedSource={failedSource}
                    stalledSource={stalledSource}
                    error={error}
                    onConnect={connectAll}
                    onRetry={retry}
                    onContinue={() => setMode('app')}
                />
            </>
        );
    }

    return (
        <>
            {banner}
            <QueryClientProvider client={queryClient}>
                <RouterProvider router={router} />
            </QueryClientProvider>
        </>
    );
}

export default App;
