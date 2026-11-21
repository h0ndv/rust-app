import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';

import { api, type CourseGroupDto } from '@/lib/api';
import { formatDate } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

function CourseCard({ group }: { group: CourseGroupDto }) {
    return (
        <article className="flex flex-col gap-3 rounded-xl border border-border p-4">
            <header className="flex flex-col gap-1">
                <span className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                    {group.code}
                </span>
                <h3 className="text-base font-semibold">{group.name}</h3>
            </header>

            <ul className="flex flex-col gap-2">
                {group.sections.map((section) => (
                    <li
                        key={section.id}
                        className="flex flex-wrap items-center gap-2 text-sm"
                    >
                        <span className="rounded bg-muted px-2 py-0.5 text-xs font-medium">
                            {section.section ?? 'Sección'}
                        </span>
                        <span className="min-w-0 flex-1 truncate text-muted-foreground">
                            {section.teacher ?? section.fullname}
                        </span>
                        {section.pending_tasks > 0 ? (
                            <span className="text-xs text-muted-foreground">
                                {section.pending_tasks} pendientes
                            </span>
                        ) : null}
                        {section.next_due_at ? (
                            <span className="text-xs text-muted-foreground">
                                {formatDate(section.next_due_at)}
                            </span>
                        ) : null}
                    </li>
                ))}
            </ul>

            <Link
                to="/ramos/$groupId"
                params={{ groupId: group.id }}
                className="text-sm text-primary underline-offset-4 hover:underline"
            >
                Ver detalle
            </Link>
        </article>
    );
}

export function Courses({ groups }: { groups: CourseGroupDto[] }) {
    if (groups.length === 0) {
        return <EmptyNote>Todavía no hay ramos sincronizados.</EmptyNote>;
    }

    return (
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {groups.map((group) => (
                <CourseCard key={group.id} group={group} />
            ))}
        </div>
    );
}

export function CoursesScreen() {
    const { data, error, isPending } = useQuery({
        queryKey: ['courses'],
        queryFn: api.listCourses,
    });

    return (
        <Screen
            title="Ramos"
            subtitle="Todas las secciones, tengan o no aula virtual."
        >
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <Courses groups={data ?? []} />
            )}
        </Screen>
    );
}
