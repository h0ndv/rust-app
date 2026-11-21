import { useTranslation } from 'react-i18next';

import type { SessionState } from '@/lib/api';
import { SOURCE_LABEL } from '@/lib/sources';

import { useSession } from './use-session';

type BannerContent = {
    title: string;
    body: string;
    action?: { label: string; onClick: () => void };
};

type BannerKey = Exclude<SessionState['state'], 'connected' | 'connecting'>;

const CONNECTING = {
    title: 'session.connectingTitle',
    body: 'session.connectingBody',
} as const;

const BANNERS = {
    expired: {
        title: 'session.expiredTitle',
        body: 'session.expiredBody',
        action: 'session.reconnect',
    },
    partial: {
        title: 'session.partialTitle',
        body: 'session.partialBody',
        action: 'session.reconnect',
    },
    disconnected: {
        title: 'session.disconnectedTitle',
        body: 'session.disconnectedBody',
        action: 'session.connect',
    },
} as const satisfies Record<
    BannerKey,
    { title: string; body: string; action: string }
>;

export function SessionBanner() {
    const { t } = useTranslation();
    const { state, connecting, connect } = useSession();

    if (state === null) {
        return null;
    }

    if (connecting || state.state === 'connecting') {
        return <Banner title={t(CONNECTING.title)} body={t(CONNECTING.body)} />;
    }

    if (state.state === 'connected') {
        return null;
    }

    const banner = BANNERS[state.state];
    const source = state.state === 'expired' ? state.source : undefined;

    return (
        <Banner
            title={t(banner.title)}
            body={t(banner.body, {
                source: source ? SOURCE_LABEL[source] : '',
            })}
            action={{
                label: t(banner.action),
                onClick: () => void connect(source),
            }}
        />
    );
}

function Banner({ title, body, action }: BannerContent) {
    return (
        <div
            role="status"
            className="mb-6 flex flex-wrap items-center gap-3 rounded-lg border border-primary/30 bg-primary/5 p-3 text-sm"
        >
            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <span className="font-medium">{title}</span>
                <span className="text-xs text-muted-foreground">{body}</span>
            </div>
            {action ? (
                <button
                    type="button"
                    className="rounded-md bg-primary px-3 py-1 text-sm text-primary-foreground"
                    onClick={action.onClick}
                >
                    {action.label}
                </button>
            ) : null}
        </div>
    );
}
