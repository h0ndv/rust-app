-- Initial schema of rust-app v3.
-- No foreign keys: sync writes each source before the union,
-- so a child row may arrive before its course. Integrity is
-- guaranteed by the upsert order.

CREATE TABLE course_groups (
    id TEXT PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE courses (
    id TEXT PRIMARY KEY,
    group_id TEXT,
    source TEXT NOT NULL,
    moodle_id INTEGER,
    banner_crn TEXT,
    shortname TEXT,
    fullname TEXT NOT NULL,
    section TEXT,
    term TEXT,
    teacher TEXT,
    moodle_url TEXT,
    banner_url TEXT,
    last_synced_at TEXT
);

CREATE TABLE sections (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    name TEXT NOT NULL,
    position INTEGER NOT NULL
);

CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    section_id TEXT,
    cmid INTEGER,
    name TEXT NOT NULL,
    modtype TEXT NOT NULL,
    url TEXT,
    due_at INTEGER,
    open_at INTEGER,
    cutoff_at INTEGER,
    status TEXT,
    submitted INTEGER,
    graded INTEGER,
    grade REAL,
    max_grade REAL,
    description TEXT NOT NULL DEFAULT '',
    content_hash TEXT,
    first_seen_at TEXT,
    acknowledged_at TEXT,
    archived_at TEXT
);

CREATE TABLE files (
    id TEXT PRIMARY KEY,
    activity_id TEXT,
    course_id TEXT,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    mimetype TEXT,
    size INTEGER,
    kind TEXT NOT NULL DEFAULT 'archivo',
    sha256 TEXT,
    local_path TEXT,
    first_seen_at TEXT,
    acknowledged_at TEXT,
    archived_at TEXT
);

CREATE TABLE grade_items (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    term TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    grade REAL,
    max_grade REAL,
    weight REAL,
    contribution REAL,
    source TEXT NOT NULL,
    raw_json TEXT
);

CREATE TABLE grade_totals (
    course_id TEXT NOT NULL,
    term TEXT NOT NULL,
    total REAL,
    max_total REAL,
    source TEXT NOT NULL,
    updated_at TEXT,
    PRIMARY KEY (course_id, term, source)
);

CREATE TABLE schedule_events (
    id TEXT PRIMARY KEY,
    course_id TEXT,
    title TEXT NOT NULL,
    day_of_week INTEGER,
    start_time TEXT,
    end_time TEXT,
    room TEXT,
    building TEXT,
    instructor TEXT,
    term TEXT,
    raw_json TEXT
);

CREATE TABLE announcements (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    url TEXT,
    published_at INTEGER,
    first_seen_at TEXT,
    acknowledged_at TEXT
);

CREATE TABLE sync_runs (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL,
    requests INTEGER NOT NULL DEFAULT 0,
    errors_json TEXT
);

CREATE TABLE source_status (
    source TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    last_checked_at TEXT,
    last_error TEXT
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE events (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    acknowledged_at TEXT
);

CREATE TABLE course_links (
    moodle_course_id TEXT NOT NULL,
    banner_course_id TEXT NOT NULL,
    PRIMARY KEY (moodle_course_id, banner_course_id)
);

CREATE VIRTUAL TABLE search_index USING fts5(kind, entity_id, title, body);

CREATE INDEX idx_courses_group ON courses(group_id);
CREATE INDEX idx_sections_course ON sections(course_id);
CREATE INDEX idx_activities_course ON activities(course_id);
CREATE INDEX idx_activities_due ON activities(due_at);
CREATE INDEX idx_activities_first_seen ON activities(first_seen_at);
CREATE INDEX idx_activities_archived ON activities(archived_at);
CREATE INDEX idx_files_course ON files(course_id);
CREATE INDEX idx_files_first_seen ON files(first_seen_at);
CREATE INDEX idx_files_archived ON files(archived_at);
CREATE INDEX idx_grade_items_course ON grade_items(course_id);
CREATE INDEX idx_schedule_events_course ON schedule_events(course_id);
CREATE INDEX idx_announcements_course ON announcements(course_id);
CREATE INDEX idx_announcements_first_seen ON announcements(first_seen_at);
CREATE INDEX idx_events_entity ON events(entity_id);
