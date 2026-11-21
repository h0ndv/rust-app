import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';

import App from './App';

vi.mock('@/lib/api', () => ({
    api: {
        hasLocalData: vi.fn().mockResolvedValue(false),
        sessionState: vi.fn().mockResolvedValue({ state: 'disconnected' }),
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
