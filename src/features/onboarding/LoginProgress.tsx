import { Check, Circle, Loader2, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import type { Source } from '@/lib/api';

export type StepStatus = 'pending' | 'active' | 'done' | 'failed';

export const STEP_LABELS: Record<Source, string> = {
    ellucian: 'Ellucian',
    moodle: 'Moodle',
    banner: 'Banner',
};

export const STEP_ORDER: Source[] = ['ellucian', 'moodle', 'banner'];

const STATUS_KEY = {
    pending: 'onboarding.steps.pending',
    active: 'onboarding.steps.active',
    done: 'onboarding.steps.done',
    failed: 'onboarding.steps.failed',
} as const;

interface LoginProgressProps {
    steps: Record<Source, StepStatus>;
    stalledSource?: Source;
}

export function LoginProgress({ steps, stalledSource }: LoginProgressProps) {
    const { t } = useTranslation();

    return (
        <ol
            className="flex flex-col gap-3"
            aria-label={t('onboarding.steps.aria')}
        >
            {STEP_ORDER.map((source) => {
                const status = steps[source] ?? 'pending';
                const stalled = stalledSource === source;

                return (
                    <li
                        key={source}
                        className="flex items-center gap-3 text-sm"
                        data-status={stalled ? 'stalled' : status}
                    >
                        <span className="flex size-5 items-center justify-center">
                            {stalled || status === 'failed' ? (
                                <X className="size-4 text-destructive" />
                            ) : status === 'done' ? (
                                <Check className="size-4 text-emerald-600" />
                            ) : status === 'active' ? (
                                <Loader2 className="size-4 animate-spin text-muted-foreground" />
                            ) : (
                                <Circle className="size-3 text-muted-foreground/50" />
                            )}
                        </span>
                        <span className="min-w-24 font-medium">
                            {STEP_LABELS[source]}
                        </span>
                        <span className="text-muted-foreground">
                            {stalled
                                ? t('onboarding.steps.stalled')
                                : t(STATUS_KEY[status])}
                        </span>
                    </li>
                );
            })}
        </ol>
    );
}
