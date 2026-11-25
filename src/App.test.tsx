import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';

import App from './App';
import { api } from './lib/api';

vi.mock('@/lib/api', () => ({
    api: {
        hasLocalData: vi.fn().mockResolvedValue(false),
        sessionState: vi.fn().mockResolvedValue({ state: 'disconnected' }),
        recoveryNotice: vi.fn().mockResolvedValue(null),
        refreshSession: vi.fn(),
        connect: vi.fn(),
        setLanguage: vi.fn(),
    },
}));

test('starts on the welcome screen', async () => {
    render(<App />);

    expect(await screen.findByText('rust-app')).toBeInTheDocument();
    expect(
        await screen.findByRole('button', { name: /connect with microsoft/i }),
    ).toBeInTheDocument();
});

test('shows the recovery banner on onboarding', async () => {
    vi.mocked(api.recoveryNotice).mockResolvedValueOnce({
        backup_path: 'C:\\panel.db.corrupt-1',
        reason: 'not a database',
    });

    render(<App />);

    expect(await screen.findByText('Local data was reset')).toBeInTheDocument();
    expect(screen.getByText(/panel\.db\.corrupt-1/)).toBeInTheDocument();
});
