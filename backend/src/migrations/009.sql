CREATE TABLE import_item_sources(item_id TEXT NOT NULL REFERENCES import_items(id),source_id TEXT NOT NULL REFERENCES sources(id),position INTEGER NOT NULL,PRIMARY KEY(item_id,source_id));
INSERT INTO import_item_sources SELECT id,source_id,0 FROM import_items;
CREATE TABLE document_job_context(document_id TEXT PRIMARY KEY REFERENCES documents(id),vacancy_id TEXT NOT NULL REFERENCES vacancies(id),resume_version_id TEXT REFERENCES document_versions(id));
INSERT INTO schema_migrations VALUES(9,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=9;
