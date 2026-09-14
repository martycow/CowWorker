ALTER TABLE vacancies ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
ALTER TABLE vacancies ADD COLUMN structured TEXT NOT NULL DEFAULT '{}';
CREATE TABLE documents_new (
 id TEXT PRIMARY KEY, title TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('resume','cover-letter','note','job-offer','agreement','tax-related','other','job-description','portfolio','reference','unknown')),
 revision INTEGER NOT NULL DEFAULT 1
);
INSERT INTO documents_new(id,title,kind) SELECT id,title,kind FROM documents;
DROP TABLE documents;
ALTER TABLE documents_new RENAME TO documents;
CREATE TABLE sources(id TEXT PRIMARY KEY, file_name TEXT NOT NULL UNIQUE, name TEXT NOT NULL, media_type TEXT NOT NULL, sha256 TEXT NOT NULL, byte_length INTEGER NOT NULL, source_url TEXT, created_at TEXT NOT NULL);
CREATE INDEX sources_hash ON sources(sha256);
CREATE TABLE source_snapshots(id TEXT PRIMARY KEY, source_id TEXT NOT NULL REFERENCES sources(id), tool TEXT NOT NULL, tool_version TEXT NOT NULL, text TEXT NOT NULL, anchors TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TABLE entity_sources(entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, source_id TEXT NOT NULL REFERENCES sources(id), PRIMARY KEY(entity_type,entity_id,source_id));
CREATE TABLE field_candidates(id TEXT PRIMARY KEY, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, field TEXT NOT NULL, value TEXT NOT NULL, raw_value TEXT NOT NULL, source_id TEXT REFERENCES sources(id), confidence REAL, operation_id TEXT, accepted INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL);
CREATE INDEX candidate_entity ON field_candidates(entity_type,entity_id,field,accepted);
CREATE TABLE field_overrides(entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, field TEXT NOT NULL, value TEXT NOT NULL, revision INTEGER NOT NULL, PRIMARY KEY(entity_type,entity_id,field));
CREATE TABLE change_proposals(id TEXT PRIMARY KEY, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, base_revision INTEGER NOT NULL, base_version TEXT, fingerprint TEXT, fields TEXT NOT NULL, source_id TEXT REFERENCES sources(id), operation_id TEXT, status TEXT NOT NULL DEFAULT 'review', created_at TEXT NOT NULL);
INSERT INTO field_overrides SELECT 'vacancy',v.id,j.key,json_quote(j.value),1 FROM vacancies v, json_each(json_object('title',title,'company',company,'location',location,'workMode',work_mode,'sourceUrl',source_url,'description',description,'notes',notes,'status',status)) j;
INSERT INTO field_overrides SELECT 'document',d.id,j.key,json_quote(j.value),1 FROM documents d, json_each(json_object('title',title,'kind',kind)) j;
CREATE TRIGGER vacancy_initial_ownership AFTER INSERT ON vacancies BEGIN
 INSERT INTO field_overrides SELECT 'vacancy',NEW.id,j.key,json_quote(j.value),1 FROM json_each(json_object('title',NEW.title,'company',NEW.company,'location',NEW.location,'workMode',NEW.work_mode,'sourceUrl',NEW.source_url,'description',NEW.description,'notes',NEW.notes,'status',NEW.status)) j;
END;
INSERT INTO schema_migrations VALUES(3,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=3;
