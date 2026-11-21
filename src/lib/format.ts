import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import { LANGUAGES } from '@/i18n';

export function formatDateFor(
    epoch: number | null | undefined,
    locale: string,
    fallback: string,
): string {
    if (!epoch) {
        return fallback;
    }
    return new Intl.DateTimeFormat(locale, {
        day: '2-digit',
        month: 'short',
        hour: '2-digit',
        minute: '2-digit',
    }).format(new Date(epoch * 1000));
}

export function formatGradeFor(
    value: number | null | undefined,
    locale: string,
    fallback: string,
): string {
    if (value === null || value === undefined) {
        return fallback;
    }
    return new Intl.NumberFormat(locale, {
        minimumFractionDigits: 1,
        maximumFractionDigits: 1,
    }).format(value);
}

export function formatSizeFor(
    bytes: number | null | undefined,
    locale: string,
): string {
    if (!bytes) {
        return '';
    }
    const number = new Intl.NumberFormat(locale, {
        maximumFractionDigits: 1,
    });
    if (bytes < 1024) {
        return `${number.format(bytes)} B`;
    }
    if (bytes < 1024 * 1024) {
        return `${number.format(bytes / 1024)} KB`;
    }
    return `${number.format(bytes / (1024 * 1024))} MB`;
}

export function weekdaysFor(locale: string): string[] {
    const formatter = new Intl.DateTimeFormat(locale, { weekday: 'long' });
    return Array.from({ length: 7 }, (_, index) =>
        formatter.format(new Date(2024, 0, 1 + index)),
    );
}

export function useFormatters() {
    const { t, i18n } = useTranslation();
    const locale =
        LANGUAGES.find((language) => language.code === i18n.language)?.locale ??
        'en-US';

    return useMemo(
        () => ({
            formatDate: (epoch: number | null | undefined) =>
                formatDateFor(epoch, locale, t('common.noDate')),
            formatGrade: (value: number | null | undefined) =>
                formatGradeFor(value, locale, t('common.notPublished')),
            formatSize: (bytes: number | null | undefined) =>
                formatSizeFor(bytes, locale),
            weekdays: weekdaysFor(locale),
        }),
        [locale, t],
    );
}
