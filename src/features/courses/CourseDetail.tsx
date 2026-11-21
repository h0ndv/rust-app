import { useQuery } from '@tanstack/react-query';
import { Link, useParams } from '@tanstack/react-router';

import { api, type CourseDetailDto } from '@/lib/api';
import { formatDate, formatGrade } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function CourseDetail({ detail }: { detail: CourseDetailDto }) {
    return (
        <div className="flex flex-col gap-8">
            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">Secciones</h3>
                <ul className="flex flex-wrap gap-2 text-sm">
                    {detail.group.sections.map((section) => (
                        <li
                            key={section.id}
                            className="rounded-md border border-border px-3 py-1"
                        >
                            {section.section ?? 'Sin sección'} ·{' '}
                            {section.teacher ?? 'Sin profesor'}
                        </li>
                    ))}
                </ul>
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">Tareas</h3>
                {detail.tasks.length === 0 ? (
                    <EmptyNote>Sin tareas registradas.</EmptyNote>
                ) : (
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {detail.tasks.map((task) => (
                            <li
                                key={task.id}
                                className="flex flex-wrap items-center gap-3 p-3 text-sm"
                            >
                                <span className="min-w-0 flex-1 truncate">
                                    {task.name}
                                </span>
                                <span className="text-xs text-muted-foreground">
                                    {task.status ?? 'sin datos'}
                                </span>
                                <span className="text-xs text-muted-foreground">
                                    {formatDate(task.due_at)}
                                </span>
                            </li>
                        ))}
                    </ul>
                )}
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">Recursos</h3>
                {detail.resources.length === 0 ? (
                    <EmptyNote>Sin recursos descargados.</EmptyNote>
                ) : (
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {detail.resources.map((resource) => (
                            <li
                                key={resource.id}
                                className="flex flex-wrap items-center gap-3 p-3 text-sm"
                            >
                                <span className="min-w-0 flex-1 truncate">
                                    {resource.name}
                                </span>
                                <button
                                    type="button"
                                    className="text-xs text-primary underline-offset-4 hover:underline"
                                    onClick={() => api.openInApp(resource.url)}
                                >
                                    Abrir
                                </button>
                            </li>
                        ))}
                    </ul>
                )}
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">Notas</h3>
                {detail.grades.length === 0 ? (
                    <EmptyNote>Sin notas publicadas.</EmptyNote>
                ) : (
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {detail.grades.map((grade) => (
                            <li
                                key={grade.id}
                                className="flex items-center gap-3 p-3 text-sm"
                            >
                                <span className="min-w-0 flex-1 truncate">
                                    {grade.name}
                                </span>
                                <span className="text-xs text-muted-foreground uppercase">
                                    {grade.source}
                                </span>
                                <span className="font-medium">
                                    {formatGrade(grade.grade)}
                                </span>
                            </li>
                        ))}
                    </ul>
                )}
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">Horario</h3>
                {detail.schedule.length === 0 ? (
                    <EmptyNote>Sin bloques de horario.</EmptyNote>
                ) : (
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {detail.schedule.map((block) => (
                            <li key={block.id} className="p-3 text-sm">
                                {block.title} · {block.start_time} -{' '}
                                {block.end_time} · {block.room ?? 'sin sala'}
                            </li>
                        ))}
                    </ul>
                )}
            </section>
        </div>
    );
}

export function CourseDetailScreen() {
    const { groupId } = useParams({ strict: false }) as { groupId: string };
    const { data, error, isPending } = useQuery({
        queryKey: ['course', groupId],
        queryFn: () => api.getCourse(groupId),
    });

    return (
        <Screen
            title={data?.group.name ?? 'Ramo'}
            subtitle={data?.group.code}
            actions={
                <Link
                    to="/ramos"
                    className="text-sm text-primary underline-offset-4 hover:underline"
                >
                    Volver a Ramos
                </Link>
            }
        >
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : data ? (
                <CourseDetail detail={data} />
            ) : null}
        </Screen>
    );
}
