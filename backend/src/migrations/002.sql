CREATE TABLE documents_new (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('resume','cover-letter','note','job-offer','agreement','tax-related','other'))
);
INSERT INTO documents_new SELECT id,title,kind FROM documents;
DROP TABLE documents;
ALTER TABLE documents_new RENAME TO documents;
CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
INSERT INTO schema_migrations VALUES (2, strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=2;
