CREATE TABLE background_tasks (
 id TEXT PRIMARY KEY, kind TEXT NOT NULL, payload_version INTEGER NOT NULL DEFAULT 1, payload TEXT NOT NULL,
 idempotency_key TEXT NOT NULL UNIQUE, status TEXT NOT NULL DEFAULT 'queued', stage TEXT NOT NULL DEFAULT 'Queued',
 progress INTEGER NOT NULL DEFAULT 0, result TEXT, error TEXT, waiting_reason TEXT,
 created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, next_attempt INTEGER NOT NULL DEFAULT 0,
 cancel_requested INTEGER NOT NULL DEFAULT 0, generation INTEGER NOT NULL DEFAULT 0, owner TEXT, lease_until INTEGER,
 attempts INTEGER NOT NULL DEFAULT 0, revision INTEGER NOT NULL DEFAULT 1, entity_id TEXT
);
CREATE INDEX task_status_due ON background_tasks(status,next_attempt,created_at);
CREATE INDEX task_entity ON background_tasks(entity_id);
CREATE TABLE task_attempts(id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES background_tasks(id), generation INTEGER NOT NULL, started_at INTEGER NOT NULL, finished_at INTEGER, status TEXT NOT NULL, error TEXT, UNIQUE(task_id,generation));
CREATE TABLE task_dependencies(task_id TEXT NOT NULL REFERENCES background_tasks(id), depends_on TEXT NOT NULL REFERENCES background_tasks(id), required INTEGER NOT NULL, PRIMARY KEY(task_id,depends_on), CHECK(task_id!=depends_on));
CREATE TABLE task_checkpoints(task_id TEXT NOT NULL REFERENCES background_tasks(id), stage TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY(task_id,stage));
CREATE TABLE runner_leases(id INTEGER PRIMARY KEY CHECK(id=1), owner TEXT NOT NULL, expires_at INTEGER NOT NULL);
INSERT INTO schema_migrations VALUES(4,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=4;
