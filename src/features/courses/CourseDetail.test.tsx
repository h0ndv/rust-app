import { render, screen } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';

import type { CourseDetailDto } from '@/lib/api';

import { CourseDetail } from './CourseDetail';

const detail = {
    group: { id: 'g1', code: 'IGR-001', name: 'Mineria', sections: [] },
    tasks: [{ id: 'a1', name: 'Laboratorio 1' }],
    resources: [{ id: 'f1', name: 'Guia.pdf' }],
    grades: [],
    schedule: [],
} as unknown as CourseDetailDto;

const scroll = vi.fn();

beforeEach(() => {
    scroll.mockClear();
    Element.prototype.scrollIntoView = scroll;
});

it('marks the focused task', () => {
    render(<CourseDetail detail={detail} focus="a1" />);

    expect(screen.getByText('Laboratorio 1').closest('li')).toHaveAttribute(
        'aria-current',
        'true',
    );
    expect(scroll).toHaveBeenCalled();
});

it('focuses a file row', () => {
    render(<CourseDetail detail={detail} focus="f1" />);

    expect(screen.getByText('Guia.pdf').closest('li')).toHaveAttribute(
        'aria-current',
        'true',
    );
});
