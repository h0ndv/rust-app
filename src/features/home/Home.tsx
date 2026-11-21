import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';

import { api, type DashboardDto, type TaskDto } from '@/lib/api';
import { formatDate } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

function TaskList({ tasks }: { tasks: TaskDto[] }) {
    return (
        <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
            {tasks.map((task) => (
                <li key={task.id} className="flex flex-col gap-1 p-3">
                    <span className="font-medium">{task.title}</span>
                    <span className="text-xs text-muted-foreground">
                        {task.course_name} · {formatDate(task.due_at)}
                    </span>
                </li>
            ))}
        </ul>
    );
}

export function Home({ dashboard }: { dashboard: DashboardDto }) {
    const empty =
        dashboard.overdue.length === 0 &&
        dashboard.due_soon.length === 0 &&
        dashboard.next_classes.length === 0 &&
        dashboard.new_resources.length === 0;

    if (empty) {
        return (
            <EmptyNote>
                Todavía no hay datos. Sincroniza para traer los ramos y las
                tareas.
            </EmptyNote>
        );
    }

    return (
        <div className="flex flex-col gap-8">
            {dashboard.overdue.length > 0 ? (
                <section className="flex flex-col gap-2">
                    <h3 className="text-sm font-semibold">Atrasadas</h3>
                    <TaskList tasks={dashboard.overdue} />
                </section>
            ) : null}

            {dashboard.due_soon.length > 0 ? (
                <section className="flex flex-col gap-2">
                    <h3 className="text-sm font-semibold">Próximos 7 días</h3>
                    <TaskList tasks={dashboard.due_soon} />
                </section>
            ) : null}

            {dashboard.next_classes.length > 0 ? (
                <section className="flex flex-col gap-2">
                    <h3 className="text-sm font-semibold">Próximas clases</h3>
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {dashboard.next_classes.map((event) => (
                            <li
                                key={event.id}
                                className="flex flex-wrap items-center gap-2 p-3 text-sm"
                            >
                                <span className="font-medium">
                                    {event.title}
                                </span>
                                <span className="text-muted-foreground">
                                    {event.start} - {event.end}
                                </span>
                                {event.room ? (
                                    <span className="text-muted-foreground">
                                        {event.room}
                                    </span>
                                ) : null}
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}

            {dashboard.new_resources.length > 0 ? (
                <section className="flex flex-col gap-2">
                    <h3 className="text-sm font-semibold">Recursos nuevos</h3>
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {dashboard.new_resources.map((resource) => (
                            <li key={resource.id} className="p-3 text-sm">
                                {resource.title}
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}

            <Link
                to="/tareas"
                className="text-sm text-primary underline-offset-4 hover:underline"
            >
                Ver todas las tareas
            </Link>
        </div>
    );
}

export function HomeScreen() {
    const { data, error, isPending } = useQuery({
        queryKey: ['dashboard'],
        queryFn: api.getDashboard,
    });

    return (
        <Screen title="Inicio" subtitle="Qué tienes pendiente y qué viene.">
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <Home
                    dashboard={
                        data ?? {
                            overdue: [],
                            due_soon: [],
                            next_classes: [],
                            new_resources: [],
                        }
                    }
                />
            )}
        </Screen>
    );
}
