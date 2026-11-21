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
        status: 'pending',
        submitted: false,
        grade: null,
        url: null,
        is_new: false,
        ...overrides,
    };
}

test('separates overdue from pending', () => {
    render(
        <Tasks
            tasks={[
                task({ id: 'a', status: 'overdue', title: 'Overdue task' }),
                task({ id: 'b', status: 'pending', title: 'Pending task' }),
            ]}
        />,
    );

    expect(screen.getByText('Overdue')).toBeInTheDocument();
    expect(screen.getByText('Pending')).toBeInTheDocument();
    expect(screen.getByText('Overdue task')).toBeInTheDocument();
    expect(screen.getByText('Pending task')).toBeInTheDocument();
});

test('shows an empty state when there are no tasks', () => {
    render(<Tasks tasks={[]} />);

    expect(screen.getByText(/no tasks match this filter/i)).toBeInTheDocument();
});

test('shows statuses outside the main groups', () => {
    render(
        <Tasks
            tasks={[
                task({
                    status: 'informational',
                    title: 'Course guide',
                }),
            ]}
        />,
    );

    expect(screen.getByText('Informational')).toBeInTheDocument();
    expect(screen.getByText('Course guide')).toBeInTheDocument();
});
