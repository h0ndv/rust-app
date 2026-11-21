import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';

import { api, type GradesDto } from '@/lib/api';
import { formatGrade } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function Grades({ grades }: { grades: GradesDto }) {
    return (
        <div className="flex flex-col gap-8">
            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">Nota oficial</h3>
                {grades.banner.length === 0 ? (
                    <EmptyNote>Sin publicar todavía.</EmptyNote>
                ) : (
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="border-b border-border text-left text-xs tracking-wide text-muted-foreground uppercase">
                                <th className="py-2">Ramo</th>
                                <th className="py-2 text-right">Nota</th>
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
                <h3 className="text-sm font-semibold">Total Moodle</h3>
                {grades.moodle_totals.length === 0 ? (
                    <EmptyNote>Sin publicar todavía.</EmptyNote>
                ) : (
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="border-b border-border text-left text-xs tracking-wide text-muted-foreground uppercase">
                                <th className="py-2">Ramo</th>
                                <th className="py-2 text-right">Total</th>
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
                    GPA del periodo: {formatGrade(grades.gpa)}
                </p>
            ) : null}
        </div>
    );
}

export function GradesScreen() {
    const [term, setTerm] = useState<string | undefined>(undefined);
    const { data, error, isPending } = useQuery({
        queryKey: ['grades', term],
        queryFn: () => api.listGrades(term),
    });

    return (
        <Screen
            title="Notas"
            subtitle="Banner y Moodle se muestran por separado, sin promediarlos."
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
