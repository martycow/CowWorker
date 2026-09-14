CREATE TABLE companies(id TEXT PRIMARY KEY,name TEXT NOT NULL,fields TEXT NOT NULL DEFAULT '{}',notes TEXT NOT NULL DEFAULT '',revision INTEGER NOT NULL DEFAULT 1,provisional INTEGER NOT NULL DEFAULT 0,created_at TEXT NOT NULL,updated_at TEXT NOT NULL,merged_into TEXT REFERENCES companies(id));
CREATE INDEX company_name ON companies(name);
CREATE UNIQUE INDEX provisional_company ON companies(name) WHERE provisional=1 AND merged_into IS NULL;
CREATE TABLE company_aliases(company_id TEXT NOT NULL REFERENCES companies(id),alias TEXT NOT NULL,verified INTEGER NOT NULL DEFAULT 0,PRIMARY KEY(company_id,alias));
CREATE TABLE company_domains(company_id TEXT NOT NULL REFERENCES companies(id),domain TEXT NOT NULL,verified INTEGER NOT NULL DEFAULT 0,PRIMARY KEY(company_id,domain));
CREATE TABLE employment_relationships(id TEXT PRIMARY KEY,company_id TEXT NOT NULL REFERENCES companies(id),kind TEXT NOT NULL CHECK(kind IN ('past','current')),role TEXT NOT NULL,start_date TEXT,end_date TEXT,confirmed INTEGER NOT NULL DEFAULT 1);
CREATE TABLE company_history(id TEXT PRIMARY KEY,company_id TEXT NOT NULL REFERENCES companies(id),action TEXT NOT NULL,detail TEXT NOT NULL,created_at TEXT NOT NULL);
ALTER TABLE vacancies ADD COLUMN company_id TEXT REFERENCES companies(id);
CREATE INDEX vacancy_company ON vacancies(company_id);
INSERT INTO companies(id,name,provisional,created_at,updated_at) SELECT lower(hex(randomblob(16))),trim(company),1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now') FROM vacancies GROUP BY trim(company);
UPDATE vacancies SET company_id=(SELECT id FROM companies WHERE name=trim(vacancies.company) AND provisional=1);
CREATE TRIGGER vacancy_company_identity AFTER INSERT ON vacancies WHEN NEW.company_id IS NULL BEGIN
 INSERT INTO companies(id,name,provisional,created_at,updated_at) SELECT lower(hex(randomblob(16))),trim(NEW.company),1,NEW.created_at,NEW.updated_at WHERE NOT EXISTS(SELECT 1 FROM companies WHERE name=trim(NEW.company) AND provisional=1 AND merged_into IS NULL);
 UPDATE vacancies SET company_id=(SELECT id FROM companies WHERE name=trim(NEW.company) AND provisional=1 AND merged_into IS NULL) WHERE id=NEW.id;
END;
INSERT INTO schema_migrations VALUES(5,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=5;
