import { appError, ERROR_KEY } from './errors';

test('recognizes a serialized Tauri error', () => {
    expect(appError({ kind: 'network', message: 'dns' })).toEqual({
        kind: 'network',
        message: 'dns',
    });
    expect(ERROR_KEY.network).toBe('errors.network');
});

test('falls back to internal for unknown values', () => {
    expect(appError(new Error('boom')).kind).toBe('internal');
    expect(appError('boom').kind).toBe('internal');
});
