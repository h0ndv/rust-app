import { render, screen } from '@testing-library/react';

import type { GradesDto } from '@/lib/api';

import { Grades } from './Grades';

const grades: GradesDto = {
    term: '202620',
    terms: ['202620'],
    banner: [
        {
            course: 'Minería de Datos',
            name: 'Minería De Datos',
            grade: 6.5,
            max_grade: 7,
            source: 'banner',
        },
    ],
    moodle_totals: [
        {
            course: 'Minería de Datos',
            total: 5.5,
            max_total: 7,
            source: 'moodle',
        },
    ],
    gpa: null,
};

test('shows Banner grades and the Moodle total separately', () => {
    render(<Grades grades={grades} />);

    expect(screen.getByText('Nota oficial')).toBeInTheDocument();
    expect(screen.getByText('Total Moodle')).toBeInTheDocument();
    expect(screen.getByText('6,5')).toBeInTheDocument();
    expect(screen.getByText('5,5')).toBeInTheDocument();
});

test('says when a total is missing instead of inventing one', () => {
    render(<Grades grades={{ ...grades, moodle_totals: [], banner: [] }} />);

    expect(screen.getAllByText(/sin publicar/i)).toHaveLength(2);
});
