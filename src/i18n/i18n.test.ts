import i18n, { isLanguage, LANGUAGES, resolveLanguage } from './index';
import { en } from './locales/en';
import { es } from './locales/es';

function keys(value: object, prefix = ''): string[] {
    return Object.entries(value).flatMap(([key, child]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        return typeof child === 'string' ? [path] : keys(child, path);
    });
}

test('translates the same key in both languages', async () => {
    await i18n.changeLanguage('es');
    expect(i18n.t('nav.courses')).toBe('Ramos');
    await i18n.changeLanguage('en');
    expect(i18n.t('nav.courses')).toBe('Courses');
});

test('both catalogs have exactly the same keys', () => {
    expect(keys(es).sort()).toEqual(keys(en).sort());
});

test('resolves unknown languages to english', () => {
    expect(resolveLanguage('es')).toBe('es');
    expect(resolveLanguage('en')).toBe('en');
    expect(resolveLanguage('fr')).toBe('en');
    expect(resolveLanguage('')).toBe('en');
    expect(resolveLanguage(null)).toBe('en');
    expect(isLanguage('EN')).toBe(false);
});

test('every supported language has a catalog', () => {
    for (const language of LANGUAGES) {
        expect(i18n.hasResourceBundle(language.code, 'translation')).toBe(true);
    }
});

test('pluralizes pending tasks per language', async () => {
    await i18n.changeLanguage('en');
    expect(i18n.t('courses.pending', { count: 1 })).toBe('1 pending task');
    expect(i18n.t('courses.pending', { count: 3 })).toBe('3 pending tasks');
    await i18n.changeLanguage('es');
    expect(i18n.t('courses.pending', { count: 1 })).toBe('1 pendiente');
    expect(i18n.t('courses.pending', { count: 3 })).toBe('3 pendientes');
});
