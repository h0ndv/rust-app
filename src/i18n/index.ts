import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import { en } from './locales/en';
import { es } from './locales/es';

export const LANGUAGES = [
    { code: 'en', label: 'English', locale: 'en-US' },
    { code: 'es', label: 'Español', locale: 'es-CL' },
] as const;

export type Language = (typeof LANGUAGES)[number]['code'];

const CATALOGS = { en, es } as const;

export function isLanguage(value: string): value is Language {
    return LANGUAGES.some((language) => language.code === value);
}

export function resolveLanguage(value: string | null | undefined): Language {
    return value !== null && value !== undefined && isLanguage(value)
        ? value
        : 'en';
}

const resources = Object.fromEntries(
    LANGUAGES.map((language) => [
        language.code,
        { translation: CATALOGS[language.code] },
    ]),
);

void i18n.use(initReactI18next).init({
    resources,
    lng: 'en',
    fallbackLng: 'en',
    supportedLngs: LANGUAGES.map((language) => language.code),
    load: 'languageOnly',
    initAsync: false,
    interpolation: { escapeValue: false },
    react: { useSuspense: false },
    returnNull: false,
});

export default i18n;
