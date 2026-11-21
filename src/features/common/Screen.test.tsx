import { render, screen, waitFor } from '@testing-library/react';
import { vi } from 'vitest';

import i18n from '@/i18n';

import { ErrorNote } from './Screen';

test('renders a localized message from a serialized command error', async () => {
    const log = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(<ErrorNote error={{ kind: 'network', message: 'dns' }} />);

    expect(screen.getByText(/network error/i)).toBeInTheDocument();
    expect(screen.queryByText('[object Object]')).not.toBeInTheDocument();

    await i18n.changeLanguage('es');
    await waitFor(() => {
        expect(screen.getByText(/error de red/i)).toBeInTheDocument();
    });

    log.mockRestore();
});
