import { useQuery } from '@tanstack/react-query';
import { Link, useParams } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';

import { api, type CourseDetailDto } from '@/lib/api';
import { useFormatters } from '@/lib/format';
import { openInApp } from '@/lib/open';
import { TASK_STATUS_KEY, taskStatus } from '@/lib/task-status';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function CourseDetail({ detail }: { detail: CourseDetailDto }) {
    const { t } = useTranslation();
    const { formatDate, formatGrade } = useFormatters();

    return (
        <div className="flex flex-col gap-8">
            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">
                    {t('courseDetail.sections')}
                </h3>
                <ul className="flex flex-wrap gap-2 text-sm">
                    {detail.group.sections.map((section) => (
                        <li
                            key={section.id}
                            className="rounded-md border border-border px-3 py-1"
                        >
                            {section.section ?? t('courseDetail.noSection')} ·{' '}
                            {section.teacher ?? t('courseDetail.noTeacher')}
                        </li>
                    ))}
                </ul>
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">
                    {t('courseDetail.tasks')}
                </h3>
                {detail.tasks.length === 0 ? (
                    <EmptyNote>{t('courseDetail.noTasks')}</EmptyNote>
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
                                {task.acknowledged_at === null ? (
                                    <span className="rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                                        {t('common.new')}
                                    </span>
                                ) : null}
                                <span className="text-xs text-muted-foreground">
                                    {t(
                                        TASK_STATUS_KEY[
                                            taskStatus(task.status) ?? 'no_data'
                                        ],
                                    )}
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
                <h3 className="text-sm font-semibold">
                    {t('courseDetail.resources')}
                </h3>
                {detail.resources.length === 0 ? (
                    <EmptyNote>{t('courseDetail.noResources')}</EmptyNote>
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
                                    onClick={() => void openInApp(resource.url)}
                                >
                                    {t('common.open')}
                                </button>
                            </li>
                        ))}
                    </ul>
                )}
            </section>

            <section className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold">
                    {t('courseDetail.grades')}
                </h3>
                {detail.grades.length === 0 ? (
                    <EmptyNote>{t('courseDetail.noGrades')}</EmptyNote>
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
                <h3 className="text-sm font-semibold">
                    {t('courseDetail.schedule')}
                </h3>
                {detail.schedule.length === 0 ? (
                    <EmptyNote>{t('courseDetail.noSchedule')}</EmptyNote>
                ) : (
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {detail.schedule.map((block) => (
                            <li key={block.id} className="p-3 text-sm">
                                {block.title || t('schedule.classFallback')} ·{' '}
                                {block.start_time} - {block.end_time} ·{' '}
                                {block.room ?? t('courseDetail.noRoom')}
                            </li>
                        ))}
                    </ul>
                )}
            </section>
        </div>
    );
}

export function CourseDetailScreen() {
    const { t } = useTranslation();
    const { groupId } = useParams({ strict: false }) as { groupId: string };
    const { data, error, isPending } = useQuery({
        queryKey: ['course', groupId],
        queryFn: () => api.getCourse(groupId),
    });

    return (
        <Screen
            title={data?.group.name ?? t('courseDetail.fallback')}
            subtitle={data?.group.code}
            actions={
                <Link
                    to="/ramos"
                    className="text-sm text-primary underline-offset-4 hover:underline"
                >
                    {t('courseDetail.back')}
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
