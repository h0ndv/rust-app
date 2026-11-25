import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { api, type SearchResultsDto } from '@/lib/api';
import { ITEM_KIND_KEY, itemKind } from '@/lib/item-kind';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function SearchResults({ results }: { results: SearchResultsDto }) {
    const { t } = useTranslation();

    if (results.total === 0) {
        return (
            <EmptyNote>
                {t('search.noResults', { query: results.query })}
            </EmptyNote>
        );
    }

    return (
        <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
            {results.items.map((item) => (
                <li
                    key={`${item.kind}-${item.id}`}
                    className="flex flex-col gap-1 p-3"
                >
                    <div className="flex flex-wrap items-center gap-2">
                        <span className="rounded bg-muted px-2 py-0.5 text-xs font-medium uppercase">
                            {t(ITEM_KIND_KEY[itemKind(item.kind) ?? 'other'])}
                        </span>
                        <span className="text-sm font-medium">
                            {item.group_id ? (
                                <Link
                                    to="/ramos/$groupId"
                                    params={{ groupId: item.group_id }}
                                    search={
                                        item.kind === 'announcement'
                                            ? {}
                                            : { focus: item.id }
                                    }
                                    className="underline-offset-4 hover:underline"
                                    title={t('common.openCourse')}
                                >
                                    {item.title}
                                </Link>
                            ) : (
                                item.title
                            )}
                        </span>
                    </div>
                    {item.body ? (
                        <p className="text-xs text-muted-foreground">
                            {item.body}
                        </p>
                    ) : null}
                </li>
            ))}
        </ul>
    );
}

export function SearchScreen() {
    const { t } = useTranslation();
    const [input, setInput] = useState('');
    const [query, setQuery] = useState('');

    const { data, error, isPending, isFetching } = useQuery({
        queryKey: ['search', query],
        queryFn: () => api.search(query),
        enabled: query.length > 0,
    });

    return (
        <Screen title={t('search.title')} subtitle={t('search.subtitle')}>
            <form
                className="flex gap-2"
                onSubmit={(event) => {
                    event.preventDefault();
                    setQuery(input.trim());
                }}
            >
                <input
                    className="flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm"
                    placeholder={t('search.placeholder')}
                    value={input}
                    onChange={(event) => setInput(event.target.value)}
                />
                <button
                    type="submit"
                    className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground"
                >
                    {t('search.button')}
                </button>
            </form>

            {query.length === 0 ? (
                <EmptyNote>{t('search.emptyQuery')}</EmptyNote>
            ) : isPending || isFetching ? (
                <Loading />
            ) : error ? (
                <ErrorNote error={error} />
            ) : (
                <SearchResults
                    results={data ?? { query, total: 0, items: [] }}
                />
            )}
        </Screen>
    );
}
