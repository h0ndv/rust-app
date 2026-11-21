import { useQuery } from '@tanstack/react-query';

import { api, type ResourceDto } from '@/lib/api';
import { formatSize } from '@/lib/format';

import { EmptyNote, ErrorNote, Loading, Screen } from '../common/Screen';

export function Resources({ resources }: { resources: ResourceDto[] }) {
    if (resources.length === 0) {
        return <EmptyNote>Todavía no hay recursos sincronizados.</EmptyNote>;
    }

    const groups = new Map<string, ResourceDto[]>();
    for (const resource of resources) {
        const key = resource.course ?? 'Sin ramo';
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
                                <span className="text-xs text-muted-foreground uppercase">
                                    {resource.kind}
                                </span>
                                <span className="text-xs text-muted-foreground">
                                    {formatSize(resource.size)}
                                </span>
                                {resource.downloaded ? (
                                    <span className="text-xs text-emerald-600">
                                        Descargado
                                    </span>
                                ) : (
                                    <button
                                        type="button"
                                        className="text-xs text-primary underline-offset-4 hover:underline"
                                        onClick={() =>
                                            api.downloadFile(resource.id)
                                        }
                                    >
                                        Descargar
                                    </button>
                                )}
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
                </section>
            ))}
        </div>
    );
}

export function ResourcesScreen() {
    const { data, error, isPending } = useQuery({
        queryKey: ['resources'],
        queryFn: () => api.listResources('all'),
    });

    return (
        <Screen
            title="Recursos"
            subtitle="Adjuntos de Moodle, agrupados por ramo."
        >
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
