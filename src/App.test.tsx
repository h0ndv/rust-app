import { render, screen } from '@testing-library/react';

import App from './App';

test('starts on the welcome screen', () => {
    render(<App />);

    expect(screen.getByText('rust-app')).toBeInTheDocument();
    expect(
        screen.getByRole('button', { name: /conectar con microsoft/i }),
    ).toBeInTheDocument();
});
