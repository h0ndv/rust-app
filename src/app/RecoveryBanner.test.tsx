import { fireEvent, render, screen } from '@testing-library/react';
import { vi } from 'vitest';

import { RecoveryBanner } from './RecoveryBanner';

it('shows the backup path and dismisses', () => {
    const onDismiss = vi.fn();

    render(
        <RecoveryBanner
            notice={{
                backup_path: 'C:\\data\\panel.db.corrupt-1',
                reason: 'not a database',
            }}
            onDismiss={onDismiss}
        />,
    );

    expect(screen.getByText(/panel\.db\.corrupt-1/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /dismiss/i }));

    expect(onDismiss).toHaveBeenCalled();
});
