import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

import { api, type ResourceDto } from '@/lib/api';
import { useFormatters } from '@/lib/format';
import { ITEM_KIND_KEY, itemKind } from '@/lib/item-kind';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function Resources({ resources }: { resources: ResourceDto[] }) {
    const { t } = useTranslation();
    const { formatSize } = useFormatters();

    if (resources.length === 0) {
        return <EmptyNote>{t('resources.empty')}</EmptyNote>;
    }

    const groups = new Map<string, ResourceDto[]>();
    for (const resource of resources) {
        const key = resource.course ?? t('resources.noCourse');
        groups.set(key, [...(groups.get(key) ?? []), resource]);
    }

    return (
        <div className="flex flex-col gap-8">
            {[...groups.entries()].map(([course, items]) => (
                <section key={course} className="flex flex-col gap-2">
                    <h3 className="text-sm font-semibold">{course}</h3>
                    <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
                        {items.map((resource) => (
                            <li
                                key={resource.id}
                                className="flex flex-wrap items-center gap-3 p-3 text-sm"
                            >
                                <span className="min-w-0 flex-1 truncate">
                                    {resource.title}
                                </span>
                                {resource.is_new ? (
                                    <span className="rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                                        {t('common.new')}
                                    </span>
                                ) : null}
                                <span className="text-xs text-muted-foreground uppercase">
                                    {t(
                                        ITEM_KIND_KEY[
                                            itemKind(resource.kind) ?? 'other'
                                        ],
                                    )}
                                </span>
                                <span className="text-xs text-muted-foreground">
                                    {formatSize(resource.size)}
                                </span>
                                {resource.downloaded ? (
                                    <span className="text-xs text-emerald-600">
                                        {t('resources.downloaded')}
                                    </span>
                                ) : (
                                    <button
                                        type="button"
                                        className="text-xs text-primary underline-offset-4 hover:underline"
                                        onClick={() =>
                                            api.downloadFile(resource.id)
                                        }
                                    >
                                        {t('resources.download')}
                                    </button>
                                )}
                                <button
                                    type="button"
                                    className="text-xs text-primary underline-offset-4 hover:underline"
                                    onClick={() => api.openInApp(resource.url)}
                                >
                                    {t('common.open')}
                                </button>
                            </li>
                        ))}
                    </ul>
                </section>
            ))}
        </div>
    );
}

export function ResourcesScreen() {
    const { t } = useTranslation();
    const { data, error, isPending } = useQuery({
        queryKey: ['resources'],
        queryFn: () => api.listResources('all'),
    });

    return (
        <Screen title={t('resources.title')} subtitle={t('resources.subtitle')}>
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <Resources resources={data ?? []} />
            )}
        </Screen>
    );
}
