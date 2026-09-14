CREATE TABLE import_sessions(id TEXT PRIMARY KEY,created_at INTEGER NOT NULL);
CREATE TABLE import_items(id TEXT PRIMARY KEY,session_id TEXT NOT NULL REFERENCES import_sessions(id),source_id TEXT NOT NULL REFERENCES sources(id),task_id TEXT NOT NULL REFERENCES background_tasks(id),revision INTEGER NOT NULL DEFAULT 1,draft TEXT,status TEXT NOT NULL DEFAULT 'processing',target_id TEXT,target_type TEXT,created_at INTEGER NOT NULL);
CREATE TABLE stage_runs(id TEXT PRIMARY KEY,item_id TEXT NOT NULL REFERENCES import_items(id),stage TEXT NOT NULL,tool TEXT NOT NULL,tool_version TEXT NOT NULL,status TEXT NOT NULL,detail TEXT,created_at INTEGER NOT NULL);
CREATE INDEX import_status ON import_items(status,created_at);
INSERT INTO schema_migrations VALUES(7,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=7;
