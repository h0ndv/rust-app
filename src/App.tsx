import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useState } from 'react';

import { router } from './app/router';
import {
    STEP_ORDER,
    type StepStatus,
} from './features/onboarding/LoginProgress';
import {
    Onboarding,
    type OnboardingState,
} from './features/onboarding/Onboarding';
import { api, type Source } from './lib/api';

const STALL_MS = 90_000;

const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: 30_000, retry: false } },
});

const INITIAL_STEPS: Record<Source, StepStatus> = {
    ellucian: 'pending',
    moodle: 'pending',
    banner: 'pending',
};

function App() {
    const [state, setState] = useState<OnboardingState>('welcome');
    const [steps, setSteps] =
        useState<Record<Source, StepStatus>>(INITIAL_STEPS);
    const [failedSource, setFailedSource] = useState<Source>();
    const [stalledSource, setStalledSource] = useState<Source>();
    const [error, setError] = useState<string>();
    const [ready, setReady] = useState(false);

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
            setError(String(cause));
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
            setFailedSource(undefined);
            setError(undefined);
            setState('connected');
        } else {
            setState('partial');
        }
    }

    if (!ready) {
        return (
            <Onboarding
                state={state}
                steps={steps}
                failedSource={failedSource}
                stalledSource={stalledSource}
                error={error}
                onConnect={connectAll}
                onRetry={retry}
                onContinue={() => setReady(true)}
            />
        );
    }

    return (
        <QueryClientProvider client={queryClient}>
            <RouterProvider router={router} />
        </QueryClientProvider>
    );
}

export default App;
