import { useQuery } from '@tanstack/react-query';

import { api, type ScheduleEventDto } from '@/lib/api';
import { WEEKDAYS } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function Schedule({ events }: { events: ScheduleEventDto[] }) {
    if (events.length === 0) {
        return <EmptyNote>Todavía no hay horario sincronizado.</EmptyNote>;
    }

    return (
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-7">
            {WEEKDAYS.map((label, index) => {
                const day = index + 1;
                const blocks = events
                    .filter((event) => event.day === day)
                    .sort((a, b) =>
                        (a.start ?? '').localeCompare(b.start ?? ''),
                    );

                return (
                    <section
                        key={label}
                        className="flex min-h-32 flex-col gap-2 rounded-lg border border-border p-3"
                    >
                        <h3 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
                            {label}
                        </h3>
                        {blocks.length === 0 ? (
                            <span className="text-xs text-muted-foreground">
                                Sin clases
                            </span>
                        ) : (
                            blocks.map((block) => (
                                <article
                                    key={block.id}
                                    className="flex flex-col gap-1 rounded-md bg-muted/50 p-2 text-sm"
                                >
                                    <span className="leading-tight font-medium">
                                        {block.title}
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        {block.start} - {block.end}
                                    </span>
                                    <span className="text-xs text-muted-foreground">
                                        {block.room ?? 'Sin sala'}
                                    </span>
                                    {block.instructor ? (
                                        <span className="text-xs text-muted-foreground">
                                            {block.instructor}
                                        </span>
                                    ) : null}
                                </article>
                            ))
                        )}
                    </section>
                );
            })}
        </div>
    );
}

export function ScheduleScreen() {
    const { data, error, isPending } = useQuery({
        queryKey: ['schedule'],
        queryFn: api.getSchedule,
    });

    return (
        <Screen
            title="Horario"
            subtitle="La semana tal como la entrega Ellucian."
        >
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <Schedule events={data ?? []} />
            )}
        </Screen>
    );
}
