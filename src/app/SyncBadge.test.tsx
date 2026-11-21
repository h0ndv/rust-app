import { render, screen, waitFor } from '@testing-library/react';

import i18n from '@/i18n';

import { SyncBadge } from './SyncBadge';

test('updates the badge when the language changes', async () => {
    render(<SyncBadge state="ok" />);

    expect(screen.getByText('Up to date')).toBeInTheDocument();

    await i18n.changeLanguage('es');
    await waitFor(() => {
        expect(screen.getByText('Al día')).toBeInTheDocument();
    });
});
