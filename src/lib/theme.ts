import { getCurrentWindow } from '@tauri-apps/api/window';

export type ThemePreference = 'system' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';

export function normalizeTheme(
    value: string | null | undefined,
): ThemePreference {
    return value === 'light' || value === 'dark' ? value : 'system';
}

export function resolveTheme(
    value: string | null | undefined,
    systemDark: boolean,
): ResolvedTheme {
    const preference = normalizeTheme(value);
    if (preference === 'system') {
        return systemDark ? 'dark' : 'light';
    }
    return preference;
}

let preference: ThemePreference = 'system';
let media: MediaQueryList | null = null;

function systemPrefersDark(): boolean {
    return typeof window !== 'undefined' &&
        typeof window.matchMedia === 'function'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
        : false;
}

function apply(): void {
    const resolved = resolveTheme(preference, systemPrefersDark());

    document.documentElement.classList.toggle('dark', resolved === 'dark');
    document.documentElement.style.colorScheme = resolved;

    try {
        void getCurrentWindow()
            .setTheme(resolved)
            .catch(() => {});
    } catch {
        // Tests and plain Vite do not have a Tauri window.
    }
}

export function initTheme(value: string | null | undefined): void {
    preference = normalizeTheme(value);

    if (media) {
        media.removeEventListener('change', apply);
        media = null;
    }
    if (
        typeof window !== 'undefined' &&
        typeof window.matchMedia === 'function'
    ) {
        media = window.matchMedia('(prefers-color-scheme: dark)');
        media.addEventListener('change', apply);
    }

    apply();
}

export function setTheme(value: string | null | undefined): void {
    preference = normalizeTheme(value);
    apply();
}
