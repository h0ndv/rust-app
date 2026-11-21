import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

import { api, type NewsItemDto } from '@/lib/api';
import { ITEM_KIND_KEY, itemKind } from '@/lib/item-kind';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function News({ items }: { items: NewsItemDto[] }) {
    const { t } = useTranslation();

    if (items.length === 0) {
        return <EmptyNote>{t('news.empty')}</EmptyNote>;
    }

    return (
        <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
            {items.map((item) => (
                <li key={item.id} className="flex flex-col gap-1 p-4">
                    <div className="flex flex-wrap items-center gap-2">
                        <span className="rounded bg-muted px-2 py-0.5 text-xs font-medium uppercase">
                            {t(ITEM_KIND_KEY[itemKind(item.kind) ?? 'other'])}
                        </span>
                        <span className="text-sm font-medium">
                            {item.title}
                        </span>
                    </div>
                    {item.course ? (
                        <span className="text-xs text-muted-foreground">
                            {item.course}
                        </span>
                    ) : null}
                    {item.body ? (
                        <p className="text-sm text-muted-foreground">
                            {item.body}
                        </p>
                    ) : null}
                </li>
            ))}
        </ul>
    );
}

export function NewsScreen() {
    const { t } = useTranslation();
    const { data, error, isPending } = useQuery({
        queryKey: ['news'],
        queryFn: api.listNews,
    });

    return (
        <Screen title={t('news.title')} subtitle={t('news.subtitle')}>
            {isPending ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <News items={data ?? []} />
            )}
        </Screen>
    );
}
