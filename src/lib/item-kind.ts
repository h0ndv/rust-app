export type ItemKind =
    'announcement' | 'resource' | 'file' | 'activity' | 'other';

const LEGACY: Record<string, ItemKind> = {
    anuncio: 'announcement',
    recurso: 'resource',
    archivo: 'file',
};

const KNOWN: ItemKind[] = [
    'announcement',
    'resource',
    'file',
    'activity',
    'other',
];

export function itemKind(value: string | null | undefined): ItemKind | null {
    if (!value) {
        return null;
    }

    const canonical = LEGACY[value] ?? value;
    return KNOWN.includes(canonical as ItemKind)
        ? (canonical as ItemKind)
        : 'other';
}

export const ITEM_KIND_KEY = {
    announcement: 'kinds.announcement',
    resource: 'kinds.resource',
    file: 'kinds.file',
    activity: 'kinds.activity',
    other: 'kinds.other',
} as const;
