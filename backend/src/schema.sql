CREATE TABLE vacancies (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    company TEXT NOT NULL,
    location TEXT NOT NULL,
    work_mode TEXT NOT NULL CHECK(work_mode IN ('Remote','Hybrid','On-site','Unspecified')),
    source_url TEXT NOT NULL,
    description TEXT NOT NULL,
    notes TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('saved','reviewed','shortlisted','archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX vacancies_source ON vacancies(source_url) WHERE source_url != '';

CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('resume','cover-letter','note','job-offer','agreement','tax-related','other')));

CREATE TABLE document_versions (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id),
    number INTEGER NOT NULL,
    file_name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    UNIQUE(document_id, number)
);

CREATE TABLE applications (
    id TEXT PRIMARY KEY,
    vacancy_id TEXT NOT NULL UNIQUE REFERENCES vacancies(id),
    stage TEXT NOT NULL CHECK(stage IN ('preparing','applied','interview','offer','rejected','withdrawn')),
    next_action TEXT NOT NULL,
    due_date TEXT NOT NULL,
    notes TEXT NOT NULL,
    submitted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE application_documents (
    application_id TEXT NOT NULL REFERENCES applications(id),
    version_id TEXT NOT NULL REFERENCES document_versions(id),
    PRIMARY KEY(application_id, version_id)
);

CREATE TABLE application_events (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id),
    stage TEXT NOT NULL, created_at TEXT NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL);

PRAGMA user_version = 1;
