import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';

import { api, type TaskDto, type TaskFilter } from '@/lib/api';
import { formatDate } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

const GROUPS = [
    { status: 'atrasada', title: 'Atrasadas' },
    { status: 'pendiente', title: 'Pendientes' },
    { status: 'entregada', title: 'Entregadas' },
    { status: 'calificada', title: 'Calificadas' },
];

const FILTERS: { value: TaskFilter; label: string }[] = [
    { value: 'all', label: 'Todas' },
    { value: 'overdue', label: 'Atrasadas' },
    { value: 'pending', label: 'Pendientes' },
    { value: 'submitted', label: 'Entregadas' },
    { value: 'graded', label: 'Calificadas' },
];

function TaskRow({ task }: { task: TaskDto }) {
    return (
        <li className="flex flex-wrap items-center gap-3 p-3">
            <div className="flex min-w-0 flex-1 flex-col">
                <span className="truncate font-medium">{task.title}</span>
                <span className="truncate text-xs text-muted-foreground">
                    {task.course_name}
                </span>
            </div>
            <span className="text-xs text-muted-foreground">
                {formatDate(task.due_at)}
            </span>
            {task.grade !== null ? (
                <span className="text-sm font-medium">
                    {task.grade.toFixed(1).replace('.', ',')}
                </span>
            ) : null}
            {task.url ? (
                <button
                    type="button"
                    className="text-xs text-primary underline-offset-4 hover:underline"
                    onClick={() => api.openInApp(task.url as string)}
                >
                    Abrir
                </button>
            ) : null}
        </li>
    );
}

export function Tasks({ tasks }: { tasks: TaskDto[] }) {
    if (tasks.length === 0) {
        return <EmptyNote>No hay tareas con este filtro.</EmptyNote>;
    }

    return (
        <div className="flex flex-col gap-6">
            {GROUPS.map((group) => {
                const items = tasks.filter(
                    (task) => task.status === group.status,
                );
                if (items.length === 0) {
                    return null;
                }

                return (
                    <section key={group.status} className="flex flex-col gap-2">
                        <h3 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
                            {group.title}
                        </h3>
                        <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                            {items.map((task) => (
                                <TaskRow key={task.id} task={task} />
                            ))}
                        </ul>
                    </section>
                );
            })}
        </div>
    );
}

export function TasksScreen() {
    const [filter, setFilter] = useState<TaskFilter>('all');
    const { data, error, isPending } = useQuery({
        queryKey: ['tasks', filter],
        queryFn: () => api.listTasks(filter),
    });

    return (
        <Screen
            title="Tareas"
            subtitle="Lo que vence, lo entregado y lo calificado."
            actions={
                <div className="flex flex-wrap gap-2">
                    {FILTERS.map((item) => (
                        <button
                            key={item.value}
                            type="button"
                            className={
                                filter === item.value
                                    ? 'rounded-md bg-primary px-3 py-1 text-sm text-primary-foreground'
                                    : 'rounded-md border border-border px-3 py-1 text-sm hover:bg-muted'
                            }
                            onClick={() => setFilter(item.value)}
                        >
                            {item.label}
                        </button>
                    ))}
                </div>
            }
        >
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <Tasks tasks={data ?? []} />
            )}
        </Screen>
    );
}
