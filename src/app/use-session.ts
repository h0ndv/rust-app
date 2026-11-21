import { useCallback, useEffect, useState } from 'react';

import type { SessionState, Source } from '@/lib/api';
import { api } from '@/lib/api';
import { onSessionChanged } from '@/lib/events';

export function useSession() {
    const [state, setState] = useState<SessionState | null>(null);
    const [connecting, setConnecting] = useState(false);

    useEffect(() => {
        let disposed = false;
        let stop: (() => void) | undefined;

        const register = async () => {
            const unlisten = await onSessionChanged((next) => {
                if (!disposed) {
                    setState(next);
                }
            });
            if (disposed) {
                unlisten();
                return;
            }
            stop = unlisten;
        };

        void register().catch(console.error);
        void api
            .sessionState()
            .then((next) => {
                if (!disposed) {
                    setState((current) => current ?? next);
                }
            })
            .catch((error) => {
                console.error(error);
                if (!disposed) {
                    setState({ state: 'disconnected' });
                }
            });

        return () => {
            disposed = true;
            stop?.();
        };
    }, []);

    const connect = useCallback(async (source?: Source) => {
        setConnecting(true);
        try {
            const next = await api.connect(source);
            setState(next);
        } catch (error) {
            console.error(error);
        } finally {
            setConnecting(false);
        }
    }, []);

    return { state, connecting, connect };
}
