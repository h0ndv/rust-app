import { ShieldCheck } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { LanguageSelect } from '@/components/language-select';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import type { Language } from '@/i18n';
import { api, type Source } from '@/lib/api';
import { ERROR_KEY, type ErrorKind } from '@/lib/errors';

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
    error?: ErrorKind;
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
    const { t, i18n } = useTranslation();
    const merged: Record<Source, StepStatus> = { ...EMPTY_STEPS, ...steps };
    const retryTarget = failedSource ?? stalledSource;

    async function changeLanguage(language: Language): Promise<void> {
        await i18n.changeLanguage(language);
        document.documentElement.lang = language;
        void api.setLanguage(language).catch((error) => {
            console.warn('No se pudo guardar el idioma', error);
        });
    }

    if (state === 'welcome') {
        return (
            <main className="flex min-h-svh items-center justify-center bg-background p-6">
                <div className="flex w-full max-w-lg flex-col gap-6">
                    <header className="flex items-start justify-between gap-3">
                        <div className="flex flex-col gap-2">
                            <h1 className="text-3xl font-semibold tracking-tight text-primary">
                                rust-app
                            </h1>
                            <p className="text-sm text-muted-foreground">
                                {t('onboarding.tagline')}
                            </p>
                        </div>
                        <LanguageSelect
                            className="rounded-md border border-border bg-background px-2 py-1 text-sm"
                            ariaLabel={t('onboarding.language')}
                            value={i18n.language}
                            onChange={(language) =>
                                void changeLanguage(language)
                            }
                        />
                    </header>

                    <section className="flex flex-col gap-3 rounded-xl border border-border p-5">
                        <div className="flex items-center gap-2 font-medium">
                            <ShieldCheck className="size-4 text-primary" />
                            {t('onboarding.beforeStart')}
                        </div>
                        <ul className="flex list-disc flex-col gap-1 pl-5 text-sm text-muted-foreground">
                            <li>{t('onboarding.privacy')}</li>
                            <li>{t('onboarding.readOnly')}</li>
                            <li>{t('onboarding.password')}</li>
                        </ul>
                    </section>

                    <Button size="lg" onClick={onConnect}>
                        {t('onboarding.connect')}
                    </Button>
                </div>
            </main>
        );
    }

    return (
        <main className="flex min-h-svh items-center justify-center bg-background p-6">
            <div className="flex w-full max-w-lg flex-col gap-6">
                <div className="flex items-start justify-between gap-3">
                    <header className="flex flex-col gap-1">
                        <h1 className="text-2xl font-semibold tracking-tight">
                            {state === 'connected'
                                ? t('onboarding.connectedTitle')
                                : state === 'partial'
                                  ? t('onboarding.partialTitle')
                                  : t('onboarding.connectingTitle')}
                        </h1>
                        <p className="text-sm text-muted-foreground">
                            {state === 'connected'
                                ? t('onboarding.connectedBody')
                                : state === 'partial'
                                  ? t('onboarding.partialBody')
                                  : t('onboarding.connectingBody')}
                        </p>
                    </header>
                    <LanguageSelect
                        className="rounded-md border border-border bg-background px-2 py-1 text-sm"
                        ariaLabel={t('onboarding.language')}
                        value={i18n.language}
                        onChange={(language) => void changeLanguage(language)}
                    />
                </div>

                <LoginProgress steps={merged} stalledSource={stalledSource} />

                {stalledSource ? (
                    <Alert variant="error">
                        <AlertTitle>
                            {t('onboarding.stalledTitle', {
                                source: STEP_LABELS[stalledSource],
                            })}
                        </AlertTitle>
                        <AlertDescription>
                            {t('onboarding.stalledBody')}
                        </AlertDescription>
                    </Alert>
                ) : null}

                {failedSource ? (
                    <Alert variant="error">
                        <AlertTitle>
                            {t('onboarding.failedTitle', {
                                source: STEP_LABELS[failedSource],
                            })}
                        </AlertTitle>
                        <AlertDescription>
                            {t('onboarding.failedBody')}
                        </AlertDescription>
                    </Alert>
                ) : null}

                {error ? (
                    <Alert variant="error">
                        <AlertTitle>
                            {t('onboarding.unexpectedError')}
                        </AlertTitle>
                        <AlertDescription>
                            {t(ERROR_KEY[error])}
                        </AlertDescription>
                    </Alert>
                ) : null}

                <div className="flex flex-wrap gap-2">
                    {retryTarget ? (
                        <Button
                            variant="outline"
                            onClick={() => onRetry?.(retryTarget)}
                        >
                            {t('common.retry')}
                        </Button>
                    ) : null}

                    {state === 'partial' || state === 'connected' ? (
                        <Button onClick={onContinue}>
                            {t('common.continue')}
                        </Button>
                    ) : null}
                </div>
            </div>
        </main>
    );
}
