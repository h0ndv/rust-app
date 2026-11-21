import {
    formatDateFor,
    formatGradeFor,
    formatSizeFor,
    weekdaysFor,
} from './format';

test('formats grades with the locale decimal separator', () => {
    expect(formatGradeFor(6.5, 'en-US', 'Not published')).toBe('6.5');
    expect(formatGradeFor(6.5, 'es-CL', 'Sin publicar')).toBe('6,5');
    expect(formatGradeFor(null, 'en-US', 'Not published')).toBe(
        'Not published',
    );
});

test('uses the locale fallback for missing dates', () => {
    expect(formatDateFor(null, 'en-US', 'No date')).toBe('No date');
    expect(formatDateFor(1_700_000_000, 'en-US', 'No date')).not.toBe(
        'No date',
    );
});

test('formats dates with the locale month convention', () => {
    const spanish = formatDateFor(1_700_000_000, 'es-CL', '');
    const english = formatDateFor(1_700_000_000, 'en-US', '');

    expect(spanish).toContain('nov');
    expect(english).toContain('Nov');
    expect(spanish).not.toBe(english);
});

test('formats sizes and weekdays per language', () => {
    expect(formatSizeFor(1024, 'en-US')).toBe('1 KB');
    expect(weekdaysFor('en-US')[0].toLowerCase()).toBe('monday');
    expect(weekdaysFor('es-CL')[0].toLowerCase()).toBe('lunes');
});
