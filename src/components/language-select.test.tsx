import { fireEvent, render, screen } from '@testing-library/react';
import { vi } from 'vitest';

import { LanguageSelect } from './language-select';

test('lists both languages and reports the selected code', () => {
    const onChange = vi.fn();
    render(
        <LanguageSelect value="en" onChange={onChange} ariaLabel="Language" />,
    );

    expect(screen.getByRole('option', { name: 'English' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Español' })).toBeInTheDocument();
    expect(
        screen.getByRole('combobox', { name: 'Language' }),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByRole('combobox'), {
        target: { value: 'es' },
    });

    expect(onChange).toHaveBeenCalledWith('es');
});
