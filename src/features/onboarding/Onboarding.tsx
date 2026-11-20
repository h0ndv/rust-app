import { ShieldCheck } from 'lucide-react';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import type { Source } from '@/lib/api';

import { LoginProgress, STEP_LABELS, type StepStatus } from './LoginProgress';

export type OnboardingState =
    'welcome' | 'connecting' | 'partial' | 'connected' | 'error';

const EMPTY_STEPS: Record<Source, StepStatus> = {
    ellucian: 'pending',
    moodle: 'pending',
    banner: 'pending',
};

interface OnboardingProps {
    state: OnboardingState;
    steps?: Partial<Record<Source, StepStatus>>;
    failedSource?: Source;
    stalledSource?: Source;
    error?: string;
    onConnect?: () => void;
    onRetry?: (source: Source) => void;
    onContinue?: () => void;
}

export function Onboarding({
    state,
    steps,
    failedSource,
    stalledSource,
    error,
    onConnect,
    onRetry,
    onContinue,
}: OnboardingProps) {
    const merged: Record<Source, StepStatus> = { ...EMPTY_STEPS, ...steps };
    const retryTarget = failedSource ?? stalledSource;

    if (state === 'welcome') {
        return (
            <main className="flex min-h-svh items-center justify-center bg-background p-6">
                <div className="flex w-full max-w-lg flex-col gap-6">
                    <header className="flex flex-col gap-2">
                        <h1 className="text-3xl font-semibold tracking-tight text-primary">
                            rust-app
                        </h1>
                        <p className="text-sm text-muted-foreground">
                            Tus ramos, tareas, horario y notas en un solo lugar.
                        </p>
                    </header>

                    <section className="flex flex-col gap-3 rounded-xl border border-border p-5">
                        <div className="flex items-center gap-2 font-medium">
                            <ShieldCheck className="size-4 text-primary" />
                            Antes de empezar
                        </div>
                        <ul className="flex list-disc flex-col gap-1 pl-5 text-sm text-muted-foreground">
                            <li>
                                Todo se guarda en este computador. No hay
                                servidor central ni telemetría.
                            </li>
                            <li>
                                La app solo lee Moodle, Ellucian y Banner. No
                                entrega, sube ni modifica nada.
                            </li>
                            <li>
                                Tu contraseña y el MFA se escriben en la ventana
                                de Microsoft, nunca en la app.
                            </li>
                        </ul>
                    </section>

                    <Button size="lg" onClick={onConnect}>
                        Conectar con Microsoft
                    </Button>
                </div>
            </main>
        );
    }

    return (
        <main className="flex min-h-svh items-center justify-center bg-background p-6">
            <div className="flex w-full max-w-lg flex-col gap-6">
                <header className="flex flex-col gap-1">
                    <h1 className="text-2xl font-semibold tracking-tight">
                        {state === 'connected'
                            ? 'Todo listo'
                            : state === 'partial'
                              ? 'Conectado a medias'
                              : 'Conectando con Microsoft'}
                    </h1>
                    <p className="text-sm text-muted-foreground">
                        {state === 'connected'
                            ? 'Las tres fuentes respondieron.'
                            : state === 'partial'
                              ? 'Una fuente no respondió. Puedes seguir con las demás.'
                              : 'Completa tu cuenta en la ventana que se abrió.'}
                    </p>
                </header>

                <LoginProgress steps={merged} stalledSource={stalledSource} />

                {stalledSource ? (
                    <Alert variant="error">
                        <AlertTitle>
                            {STEP_LABELS[stalledSource]} no está respondiendo
                        </AlertTitle>
                        <AlertDescription>
                            Puede ser la red o la ventana de login. Reintenta
                            solo ese paso.
                        </AlertDescription>
                    </Alert>
                ) : null}

                {failedSource ? (
                    <Alert variant="error">
                        <AlertTitle>
                            {STEP_LABELS[failedSource]} no se pudo conectar
                        </AlertTitle>
                        <AlertDescription>
                            Revisa la ventana de login o reintenta el paso.
                        </AlertDescription>
                    </Alert>
                ) : null}

                {error ? (
                    <Alert variant="error">
                        <AlertTitle>Error inesperado</AlertTitle>
                        <AlertDescription>{error}</AlertDescription>
                    </Alert>
                ) : null}

                <div className="flex flex-wrap gap-2">
                    {retryTarget ? (
                        <Button
                            variant="outline"
                            onClick={() => onRetry?.(retryTarget)}
                        >
                            Reintentar
                        </Button>
                    ) : null}

                    {state === 'partial' || state === 'connected' ? (
                        <Button onClick={onContinue}>Entrar</Button>
                    ) : null}
                </div>
            </div>
        </main>
    );
}
