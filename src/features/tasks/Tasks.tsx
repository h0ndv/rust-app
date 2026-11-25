import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { api, type TaskDto, type TaskFilter } from '@/lib/api';
import { useFormatters } from '@/lib/format';
import { openInApp } from '@/lib/open';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

const GROUPS = [
    { status: 'overdue', labelKey: 'tasks.groups.overdue' as const },
    { status: 'pending', labelKey: 'tasks.groups.pending' as const },
    { status: 'submitted', labelKey: 'tasks.groups.submitted' as const },
    { status: 'graded', labelKey: 'tasks.groups.graded' as const },
    { status: 'no_data', labelKey: 'tasks.groups.no_data' as const },
    { status: 'unconfirmed', labelKey: 'tasks.groups.unconfirmed' as const },
    {
        status: 'informational',
        labelKey: 'tasks.groups.informational' as const,
    },
];

const FILTERS: {
    value: TaskFilter;
    labelKey:
        | 'tasks.filters.all'
        | 'tasks.filters.overdue'
        | 'tasks.filters.pending'
        | 'tasks.filters.submitted'
        | 'tasks.filters.graded'
        | 'tasks.filters.no_data'
        | 'tasks.filters.unconfirmed'
        | 'tasks.filters.informational';
}[] = [
    { value: 'all', labelKey: 'tasks.filters.all' },
    { value: 'overdue', labelKey: 'tasks.filters.overdue' },
    { value: 'pending', labelKey: 'tasks.filters.pending' },
    { value: 'submitted', labelKey: 'tasks.filters.submitted' },
    { value: 'graded', labelKey: 'tasks.filters.graded' },
    { value: 'no_data', labelKey: 'tasks.filters.no_data' },
    { value: 'unconfirmed', labelKey: 'tasks.filters.unconfirmed' },
    { value: 'informational', labelKey: 'tasks.filters.informational' },
];

function TaskRow({ task }: { task: TaskDto }) {
    const { t } = useTranslation();
    const { formatDate, formatGrade } = useFormatters();

    return (
        <li className="flex flex-wrap items-center gap-3 p-3">
            <div className="flex min-w-0 flex-1 flex-col">
                <span className="flex min-w-0 items-center gap-2">
                    <span className="truncate font-medium">{task.title}</span>
                    {task.is_new ? (
                        <span className="rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                            {t('common.new')}
                        </span>
                    ) : null}
                </span>
                <span className="truncate text-xs text-muted-foreground">
                    {task.course_name}
                </span>
            </div>
            <span className="text-xs text-muted-foreground">
                {formatDate(task.due_at)}
            </span>
            {task.grade !== null ? (
                <span className="text-sm font-medium">
                    {formatGrade(task.grade)}
                </span>
            ) : null}
            {task.url ? (
                <button
                    type="button"
                    className="text-xs text-primary underline-offset-4 hover:underline"
                    onClick={() => void openInApp(task.url as string)}
                >
                    {t('common.open')}
                </button>
            ) : null}
        </li>
    );
}

export function Tasks({ tasks }: { tasks: TaskDto[] }) {
    const { t } = useTranslation();

    if (tasks.length === 0) {
        return <EmptyNote>{t('tasks.empty')}</EmptyNote>;
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
                            {t(group.labelKey)}
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
    const { t } = useTranslation();
    const [filter, setFilter] = useState<TaskFilter>('all');
    const { data, error, isPending } = useQuery({
        queryKey: ['tasks', filter],
        queryFn: () => api.listTasks(filter),
    });

    return (
        <Screen
            title={t('tasks.title')}
            subtitle={t('tasks.subtitle')}
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
                            {t(item.labelKey)}
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
