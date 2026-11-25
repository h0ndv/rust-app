import { screen } from '@testing-library/react';

import type { SearchResultsDto } from '@/lib/api';

import { renderRoute } from '@/test/router';

import { SearchResults } from './Search';

const results: SearchResultsDto = {
    query: 'lab',
    total: 2,
    items: [
        {
            kind: 'activity',
            id: 'a1',
            title: 'Lab 1',
            body: '',
            group_id: 'g1',
        },
        {
            kind: 'activity',
            id: 'a2',
            title: 'Old lab',
            body: '',
            group_id: null,
        },
    ],
};

it('links a hit to its course and focus', async () => {
    await renderRoute(<SearchResults results={results} />);

    expect(screen.getByRole('link', { name: 'Lab 1' })).toHaveAttribute(
        'href',
        '/ramos/g1?focus=a1',
    );
});

it('keeps an orphan hit as plain text', async () => {
    await renderRoute(<SearchResults results={results} />);

    expect(screen.queryByRole('link', { name: 'Old lab' })).toBeNull();
    expect(screen.getByText('Old lab')).toBeInTheDocument();
});
