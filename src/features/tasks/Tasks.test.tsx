import { render, screen } from '@testing-library/react';

import type { TaskDto } from '@/lib/api';

import { Tasks } from './Tasks';

function task(overrides: Partial<TaskDto>): TaskDto {
    return {
        id: 'assign:1',
        course_id: 'moodle:1',
        course_name: 'Minería de Datos',
        title: 'Taller',
        due_at: null,
        status: 'pendiente',
        submitted: false,
        grade: null,
        url: null,
        ...overrides,
    };
}

test('separates overdue from pending', () => {
    render(
        <Tasks
            tasks={[
                task({ id: 'a', status: 'atrasada', title: 'Atrasada' }),
                task({ id: 'b', status: 'pendiente', title: 'Pendiente' }),
            ]}
        />,
    );

    expect(screen.getByText('Atrasadas')).toBeInTheDocument();
    expect(screen.getByText('Pendientes')).toBeInTheDocument();
    expect(screen.getByText('Atrasada')).toBeInTheDocument();
    expect(screen.getByText('Pendiente')).toBeInTheDocument();
});

test('shows an empty state when there are no tasks', () => {
    render(<Tasks tasks={[]} />);

    expect(screen.getByText(/no hay tareas/i)).toBeInTheDocument();
});
