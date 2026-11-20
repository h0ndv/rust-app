import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';

import { Onboarding } from './Onboarding';

test('shows the three steps and allows retrying', () => {
    render(
        <Onboarding state="partial" failedSource="banner" onRetry={vi.fn()} />,
    );

    expect(screen.getByText('Ellucian')).toBeInTheDocument();
    expect(screen.getByText('Moodle')).toBeInTheDocument();
    expect(screen.getByText('Banner')).toBeInTheDocument();
    expect(
        screen.getByRole('button', { name: /reintentar/i }),
    ).toBeInTheDocument();
});

test('shows retry when a step stalls', () => {
    render(
        <Onboarding
            state="connecting"
            stalledSource="moodle"
            onRetry={vi.fn()}
        />,
    );

    expect(screen.getByText(/no está respondiendo/i)).toBeInTheDocument();
    expect(
        screen.getByRole('button', { name: /reintentar/i }),
    ).toBeInTheDocument();
});

test('welcome explains the app only reads and stores data locally', () => {
    render(<Onboarding state="welcome" onConnect={vi.fn()} />);

    expect(screen.getByText(/no hay servidor central/i)).toBeInTheDocument();
    expect(
        screen.getByRole('button', { name: /conectar con microsoft/i }),
    ).toBeInTheDocument();
});

test('connected session lets the student in', () => {
    const onContinue = vi.fn();
    render(<Onboarding state="connected" onContinue={onContinue} />);

    screen.getByRole('button', { name: /entrar/i }).click();
    expect(onContinue).toHaveBeenCalled();
});
