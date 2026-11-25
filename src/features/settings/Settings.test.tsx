import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { vi } from 'vitest';

import i18n from '@/i18n';
import { api } from '@/lib/api';

import { SettingsScreen } from './Settings';

vi.mock('@/lib/api', () => ({
    api: {
        getSettings: vi.fn(),
        saveSettings: vi.fn(),
        clearData: vi.fn(),
        exportData: vi.fn(),
        clearDownloads: vi.fn(),
    },
}));

const settings = {
    sync_interval_minutes: 30,
    downloads_dir: 'C:\\Downloads',
    theme: 'system',
    notifications: false,
    language: 'en',
};

test('saves the selected language and switches the interface', async () => {
    vi.mocked(api.getSettings).mockResolvedValue(settings);
    vi.mocked(api.saveSettings).mockResolvedValue(undefined);
    vi.mocked(api.clearData).mockResolvedValue(0);

    const client = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });

    render(
        <QueryClientProvider client={client}>
            <SettingsScreen />
        </QueryClientProvider>,
    );

    const select = await screen.findByRole('combobox', {
        name: /language/i,
    });
    fireEvent.change(select, { target: { value: 'es' } });
    fireEvent.click(screen.getByRole('button', { name: /save/i }));

    await waitFor(() => {
        expect(api.saveSettings).toHaveBeenCalledWith(
            expect.objectContaining({ language: 'es' }),
        );
    });

    await waitFor(() => {
        expect(i18n.language).toBe('es');
    });

    expect(await screen.findByText('Ajustes guardados.')).toBeInTheDocument();
    expect(client.getQueryData(['settings'])).toMatchObject({
        language: 'es',
    });
});

test('exports the data and shows the path', async () => {
    vi.mocked(api.getSettings).mockResolvedValue(settings);
    vi.mocked(api.exportData).mockResolvedValue(
        'C:\\Downloads\\rust-app-export.json',
    );

    const client = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });

    render(
        <QueryClientProvider client={client}>
            <SettingsScreen />
        </QueryClientProvider>,
    );

    fireEvent.click(
        await screen.findByRole('button', { name: /export data/i }),
    );

    await waitFor(() => {
        expect(api.exportData).toHaveBeenCalled();
    });
    expect(
        await screen.findByText(/rust-app-export\.json/),
    ).toBeInTheDocument();
});

test('clears downloaded files after confirmation', async () => {
    vi.mocked(api.getSettings).mockResolvedValue(settings);
    vi.mocked(api.clearDownloads).mockResolvedValue(2);
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);

    const client = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });

    render(
        <QueryClientProvider client={client}>
            <SettingsScreen />
        </QueryClientProvider>,
    );

    fireEvent.click(
        await screen.findByRole('button', {
            name: /clear downloaded files/i,
        }),
    );

    await waitFor(() => {
        expect(api.clearDownloads).toHaveBeenCalled();
    });
    expect(await screen.findByText(/2 files deleted/i)).toBeInTheDocument();
    expect(confirm).toHaveBeenCalled();

    confirm.mockRestore();
});

test('does not clear downloads when cancelled', async () => {
    vi.mocked(api.getSettings).mockResolvedValue(settings);
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);

    const client = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });

    render(
        <QueryClientProvider client={client}>
            <SettingsScreen />
        </QueryClientProvider>,
    );

    fireEvent.click(
        await screen.findByRole('button', {
            name: /clear downloaded files/i,
        }),
    );

    expect(api.clearDownloads).not.toHaveBeenCalled();
    confirm.mockRestore();
});
