import { invoke } from '@tauri-apps/api/core';

export type Source = 'moodle' | 'ellucian' | 'banner';

/** Mirrors `SessionState` in Rust, tagged with `state` and optional `source`. */
export type SessionState =
    | { state: 'disconnected' }
    | { state: 'connecting'; source: Source }
    | { state: 'connected' }
    | { state: 'expired'; source: Source }
    | { state: 'partial' };

export type SyncStatusKind = 'running' | 'ok' | 'partial' | 'failed';

export interface SyncRun {
    id: string;
    status: SyncStatusKind;
    requests: number;
    started_at: string;
    finished_at: string | null;
}

export interface SyncStatus {
    state: SyncStatusKind;
    source: Source | null;
    progress: number;
    requests: number;
    errors: string[];
}

export interface TaskDto {
    id: string;
    course_id: string;
    course_name: string;
    title: string;
    due_at: number | null;
    status: string | null;
    submitted: boolean;
    grade: number | null;
    url: string | null;
}

export interface ResourceDto {
    id: string;
    course_id: string | null;
    title: string;
    kind: string;
    mimetype: string | null;
    size: number | null;
    url: string;
    is_new: boolean;
    downloaded: boolean;
}

export interface ScheduleEventDto {
    id: string;
    course: string | null;
    title: string;
    day: number | null;
    start: string | null;
    end: string | null;
    room: string | null;
    building: string | null;
    instructor: string | null;
}

export interface NewsItemDto {
    id: string;
    course: string | null;
    title: string;
    body: string;
    published_at: number | null;
    kind: string;
}

export interface DashboardDto {
    overdue: TaskDto[];
    due_soon: TaskDto[];
    next_classes: ScheduleEventDto[];
    new_resources: ResourceDto[];
}

export interface CourseSectionDto {
    id: string;
    fullname: string;
    shortname: string | null;
    section: string | null;
    term: string | null;
    teacher: string | null;
    source: string;
    banner_crn: string | null;
    next_due_at: number | null;
    pending_tasks: number;
}

export interface CourseGroupDto {
    id: string;
    code: string;
    name: string;
    sections: CourseSectionDto[];
}

export interface GradeRowDto {
    course: string;
    name: string;
    grade: number | null;
    max_grade: number | null;
    source: string;
}

export interface GradeTotalDto {
    course: string;
    total: number | null;
    max_total: number | null;
    source: string;
}

export interface GradesDto {
    term: string;
    terms: string[];
    banner: GradeRowDto[];
    moodle_totals: GradeTotalDto[];
    gpa: number | null;
}

export interface SearchHitDto {
    kind: string;
    id: string;
    title: string;
    body: string;
}

export interface SearchResultsDto {
    query: string;
    total: number;
    items: SearchHitDto[];
}

export interface SettingsDto {
    sync_interval_minutes: number;
    downloads_dir: string;
    theme: string;
    notifications: boolean;
}

export type TaskFilter = 'all' | 'overdue' | 'pending' | 'submitted' | 'graded';
export type ResourceFilter = 'all' | 'new' | 'downloaded';

export const api = {
    connect: (source?: Source) =>
        invoke<SessionState>('connect', { source: source ?? null }),
    sessionState: () => invoke<SessionState>('session_state'),

    startSync: () => invoke<SyncRun>('start_sync'),
    cancelSync: () => invoke<void>('cancel_sync'),
    syncStatus: () => invoke<SyncStatus>('sync_status'),

    getDashboard: () => invoke<DashboardDto>('get_dashboard'),
    listCourses: () => invoke<CourseGroupDto[]>('list_courses'),
    getCourse: (groupId: string) => invoke<unknown>('get_course', { groupId }),
    listTasks: (filter: TaskFilter = 'all') =>
        invoke<TaskDto[]>('list_tasks', { filter }),
    listResources: (filter: ResourceFilter = 'all') =>
        invoke<ResourceDto[]>('list_resources', { filter }),
    listGrades: (term?: string) =>
        invoke<GradesDto>('list_grades', { term: term ?? null }),
    getSchedule: () => invoke<ScheduleEventDto[]>('get_schedule'),
    listNews: () => invoke<NewsItemDto[]>('list_news'),
    search: (query: string) => invoke<SearchResultsDto>('search', { query }),

    getSettings: () => invoke<SettingsDto>('get_settings'),
    saveSettings: (settings: SettingsDto) =>
        invoke<void>('save_settings', { settings }),

    openInApp: (url: string) => invoke<void>('open_in_app', { url }),
    downloadFile: (fileId: string) => invoke<void>('download_file', { fileId }),
    clearData: () => invoke<number>('clear_data'),
};
