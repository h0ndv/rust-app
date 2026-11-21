import { ITEM_KIND_KEY, itemKind } from './item-kind';
import { TASK_STATUS_KEY, taskStatus } from './task-status';

test('maps legacy and canonical task statuses', () => {
    expect(taskStatus('atrasada')).toBe('overdue');
    expect(taskStatus('pendiente')).toBe('pending');
    expect(taskStatus('overdue')).toBe('overdue');
    expect(taskStatus('desconocida')).toBeNull();
    expect(taskStatus(null)).toBeNull();
    expect(TASK_STATUS_KEY.overdue).toBe('status.overdue');
});

test('maps legacy and canonical item kinds', () => {
    expect(itemKind('anuncio')).toBe('announcement');
    expect(itemKind('recurso')).toBe('resource');
    expect(itemKind('archivo')).toBe('file');
    expect(itemKind('file')).toBe('file');
    expect(itemKind('desconocido')).toBe('other');
    expect(itemKind(null)).toBeNull();
    expect(ITEM_KIND_KEY.file).toBe('kinds.file');
});
