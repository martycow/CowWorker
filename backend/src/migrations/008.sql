CREATE TABLE research_runs(id TEXT PRIMARY KEY,company_id TEXT NOT NULL REFERENCES companies(id),task_id TEXT NOT NULL REFERENCES background_tasks(id),base_revision INTEGER NOT NULL,source_url TEXT NOT NULL,status TEXT NOT NULL DEFAULT 'queued',source_id TEXT REFERENCES sources(id),error TEXT,created_at INTEGER NOT NULL,completed_at INTEGER);
CREATE TABLE company_assets(id TEXT PRIMARY KEY,company_id TEXT NOT NULL REFERENCES companies(id),source_id TEXT NOT NULL REFERENCES sources(id),kind TEXT NOT NULL,created_at INTEGER NOT NULL);
CREATE INDEX company_asset_owner ON company_assets(company_id,kind,created_at);
INSERT INTO schema_migrations VALUES(8,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=8;
