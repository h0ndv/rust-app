import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { vi } from 'vitest';

import i18n from '@/i18n';
import { api } from '@/lib/api';

import { Onboarding } from './Onboarding';

vi.mock('@/lib/api', () => ({
    api: {
        setLanguage: vi.fn().mockResolvedValue(undefined),
    },
}));

test('shows the three steps and allows retrying', () => {
    render(
        <Onboarding state="partial" failedSource="banner" onRetry={vi.fn()} />,
    );

    expect(screen.getByText('Ellucian')).toBeInTheDocument();
    expect(screen.getByText('Moodle')).toBeInTheDocument();
    expect(screen.getByText('Banner')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
});

test('shows retry when a step stalls', () => {
    render(
        <Onboarding
            state="connecting"
            stalledSource="moodle"
            onRetry={vi.fn()}
        />,
    );

    expect(screen.getByText(/is not responding/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
});

test('welcome explains the app only reads and stores data locally', () => {
    render(<Onboarding state="welcome" onConnect={vi.fn()} />);

    expect(screen.getByText(/no central server/i)).toBeInTheDocument();
    expect(
        screen.getByRole('button', { name: /connect with microsoft/i }),
    ).toBeInTheDocument();
    expect(
        screen.getByRole('combobox', { name: /language/i }),
    ).toBeInTheDocument();
});

test('connected session lets the student in', () => {
    const onContinue = vi.fn();
    render(<Onboarding state="connected" onContinue={onContinue} />);

    screen.getByRole('button', { name: /continue/i }).click();
    expect(onContinue).toHaveBeenCalled();
});

test('persists the language chosen before login', async () => {
    render(<Onboarding state="welcome" onConnect={vi.fn()} />);

    fireEvent.change(screen.getByRole('combobox'), {
        target: { value: 'es' },
    });

    await waitFor(() => {
        expect(api.setLanguage).toHaveBeenCalledWith('es');
    });
});

test('localizes a command error', async () => {
    render(<Onboarding state="partial" error="network" />);

    expect(screen.getByText(/network error/i)).toBeInTheDocument();

    await i18n.changeLanguage('es');
    await waitFor(() => {
        expect(screen.getByText(/error de red/i)).toBeInTheDocument();
    });
});
