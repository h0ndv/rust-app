import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { api, type GradesDto } from '@/lib/api';
import { useFormatters } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function Grades({ grades }: { grades: GradesDto }) {
    const { t } = useTranslation();
    const { formatGrade } = useFormatters();

    return (
        <div className="flex flex-col gap-8">
            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">
                    {t('grades.official')}
                </h3>
                {grades.banner.length === 0 ? (
                    <EmptyNote>{t('grades.notPublishedYet')}</EmptyNote>
                ) : (
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="border-b border-border text-left text-xs tracking-wide text-muted-foreground uppercase">
                                <th className="py-2">{t('grades.course')}</th>
                                <th className="py-2 text-right">
                                    {t('grades.grade')}
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {grades.banner.map((row) => (
                                <tr
                                    key={`${row.course}-${row.name}`}
                                    className="border-b border-border/60"
                                >
                                    <td className="py-2">{row.course}</td>
                                    <td className="py-2 text-right font-medium">
                                        {formatGrade(row.grade)}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">
                    {t('grades.moodleTotal')}
                </h3>
                {grades.moodle_totals.length === 0 ? (
                    <EmptyNote>{t('grades.notPublishedYet')}</EmptyNote>
                ) : (
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="border-b border-border text-left text-xs tracking-wide text-muted-foreground uppercase">
                                <th className="py-2">{t('grades.course')}</th>
                                <th className="py-2 text-right">
                                    {t('grades.total')}
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {grades.moodle_totals.map((row) => (
                                <tr
                                    key={row.course}
                                    className="border-b border-border/60"
                                >
                                    <td className="py-2">{row.course}</td>
                                    <td className="py-2 text-right font-medium">
                                        {formatGrade(row.total)}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </section>

            {grades.gpa !== null ? (
                <p className="text-sm text-muted-foreground">
                    {t('grades.termGpa', {
                        value: formatGrade(grades.gpa),
                    })}
                </p>
            ) : null}
        </div>
    );
}

export function GradesScreen() {
    const { t } = useTranslation();
    const [term, setTerm] = useState<string | undefined>(undefined);
    const { data, error, isPending } = useQuery({
        queryKey: ['grades', term],
        queryFn: () => api.listGrades(term),
    });

    return (
        <Screen
            title={t('grades.title')}
            subtitle={t('grades.subtitle')}
            actions={
                data && data.terms.length > 1 ? (
                    <select
                        className="rounded-md border border-border bg-background px-3 py-1 text-sm"
                        value={data.term}
                        onChange={(event) => setTerm(event.target.value)}
                    >
                        {data.terms.map((item) => (
                            <option key={item} value={item}>
                                {item}
                            </option>
                        ))}
                    </select>
                ) : null
            }
        >
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <Grades
                    grades={
                        data ?? {
                            term: '',
                            terms: [],
                            banner: [],
                            moodle_totals: [],
                            gpa: null,
                        }
                    }
                />
            )}
        </Screen>
    );
}
