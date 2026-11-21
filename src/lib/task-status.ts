export type TaskStatus =
    | 'overdue'
    | 'pending'
    | 'submitted'
    | 'graded'
    | 'no_data'
    | 'unconfirmed'
    | 'informational';

const LEGACY: Record<string, TaskStatus> = {
    atrasada: 'overdue',
    pendiente: 'pending',
    entregada: 'submitted',
    calificada: 'graded',
    sin_datos: 'no_data',
    por_confirmar: 'unconfirmed',
    informativa: 'informational',
};

const KNOWN: TaskStatus[] = [
    'overdue',
    'pending',
    'submitted',
    'graded',
    'no_data',
    'unconfirmed',
    'informational',
];

export function taskStatus(
    value: string | null | undefined,
): TaskStatus | null {
    if (!value) {
        return null;
    }

    const canonical = LEGACY[value] ?? value;
    return KNOWN.includes(canonical as TaskStatus)
        ? (canonical as TaskStatus)
        : null;
}

export const TASK_STATUS_KEY = {
    overdue: 'status.overdue',
    pending: 'status.pending',
    submitted: 'status.submitted',
    graded: 'status.graded',
    no_data: 'status.no_data',
    unconfirmed: 'status.unconfirmed',
    informational: 'status.informational',
} as const;
