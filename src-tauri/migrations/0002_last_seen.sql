-- "7 days unseen" needs to know when each item was last seen.
ALTER TABLE activities ADD COLUMN last_seen_at TEXT;
ALTER TABLE files ADD COLUMN last_seen_at TEXT;

CREATE INDEX idx_activities_last_seen ON activities(last_seen_at);
CREATE INDEX idx_files_last_seen ON files(last_seen_at);
