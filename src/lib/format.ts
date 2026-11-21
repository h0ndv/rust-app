const dateFormatter = new Intl.DateTimeFormat('es-CL', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
});

export const WEEKDAYS = [
    'Lunes',
    'Martes',
    'Miércoles',
    'Jueves',
    'Viernes',
    'Sábado',
    'Domingo',
];

export function formatDate(epoch: number | null | undefined): string {
    if (!epoch) {
        return 'Sin fecha';
    }
    return dateFormatter.format(new Date(epoch * 1000));
}

export function formatGrade(value: number | null | undefined): string {
    if (value === null || value === undefined) {
        return 'Sin publicar';
    }
    return value.toFixed(1).replace('.', ',');
}

export function formatSize(bytes: number | null | undefined): string {
    if (!bytes) {
        return '';
    }
    if (bytes < 1024) {
        return `${bytes} B`;
    }
    if (bytes < 1024 * 1024) {
        return `${Math.round(bytes / 1024)} KB`;
    }
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
