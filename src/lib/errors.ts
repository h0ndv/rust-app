export const ERROR_KINDS = [
    'network',
    'unauthorized',
    'parse',
    'not_found',
    'rate_limited',
    'internal',
] as const;

export type ErrorKind = (typeof ERROR_KINDS)[number];

export interface AppErrorPayload {
    kind: ErrorKind;
    message: string;
}

export const ERROR_KEY = {
    network: 'errors.network',
    unauthorized: 'errors.unauthorized',
    parse: 'errors.parse',
    not_found: 'errors.not_found',
    rate_limited: 'errors.rate_limited',
    internal: 'errors.internal',
} as const satisfies Record<ErrorKind, string>;

export function appError(error: unknown): AppErrorPayload {
    if (typeof error === 'object' && error !== null && 'kind' in error) {
        const kind = String((error as { kind: unknown }).kind);
        if ((ERROR_KINDS as readonly string[]).includes(kind)) {
            const message =
                'message' in error
                    ? String((error as { message: unknown }).message)
                    : '';
            return { kind: kind as ErrorKind, message };
        }
    }

    return {
        kind: 'internal',
        message: error instanceof Error ? error.message : String(error),
    };
}
