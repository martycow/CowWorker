use crate::model::*;
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::{fs, path::PathBuf, time::Duration};
use uuid::Uuid;

type Result<T> = std::result::Result<T, String>;
pub struct Store {
    db: Connection,
    documents_path: PathBuf,
}

fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn now() -> String {
    Utc::now().to_rfc3339()
}
fn id() -> String {
    Uuid::new_v4().to_string()
}
fn required(value: &str, name: &str, max: usize) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        return Err(format!("{name} must contain 1 to {max} bytes."));
    }
    Ok(value.to_owned())
}
fn limited(value: &str, name: &str, max: usize) -> Result<()> {
    if value.len() > max {
        return Err(format!("{name} exceeds {max} bytes."));
    }
    Ok(())
}
fn one_of(value: &str, choices: &[&str], name: &str) -> Result<()> {
    if !choices.contains(&value) {
        return Err(format!("Invalid {name}."));
    }
    Ok(())
}

impl Store {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        fs::create_dir_all(&path).map_err(error)?;
        let documents_path = path.join("documents");
        fs::create_dir_all(&documents_path).map_err(error)?;
        let db = Connection::open(path.join("cowworker.db")).map_err(error)?;
        db.busy_timeout(Duration::from_secs(5)).map_err(error)?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")
            .map_err(error)?;
        let version: i64 = db
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(error)?;
        match version {
            0 => {
                let tx = db.unchecked_transaction().map_err(error)?;
                tx.execute_batch(include_str!("schema.sql"))
                    .map_err(error)?;
                tx.commit().map_err(error)?;
            }
            1 => (),
            _ => return Err("This workspace was created by a newer CowWorker version.".into()),
        }
        Ok(Self { db, documents_path })
    }

    pub fn workspace(&self) -> Result<Workspace> {
        Ok(Workspace {
            schema_version: 1,
            vacancies: self.vacancies()?,
            documents: self.documents()?,
            applications: self.applications()?,
            profile: self.profile()?,
        })
    }

    fn vacancies(&self) -> Result<Vec<Vacancy>> {
        let mut stmt = self.db.prepare("SELECT id,title,company,location,work_mode,source_url,description,notes,status,created_at,updated_at FROM vacancies ORDER BY created_at DESC,id").map_err(error)?;
        let records = stmt
            .query_map([], |r| {
                Ok(Vacancy {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    company: r.get(2)?,
                    location: r.get(3)?,
                    work_mode: r.get(4)?,
                    source_url: r.get(5)?,
                    description: r.get(6)?,
                    notes: r.get(7)?,
                    status: r.get(8)?,
                    created_at: r.get(9)?,
                    updated_at: r.get(10)?,
                })
            })
            .map_err(error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(error);
        records
    }

    pub fn save_vacancy(&mut self, input: VacancyInput) -> Result<String> {
        let title = required(&input.title, "Role", 300)?;
        let company = required(&input.company, "Company", 300)?;
        limited(&input.location, "Location", 500)?;
        limited(&input.description, "Description", 200_000)?;
        limited(&input.notes, "Notes", 100_000)?;
        one_of(
            &input.work_mode,
            &["Remote", "Hybrid", "On-site", "Unspecified"],
            "work mode",
        )?;
        one_of(
            &input.status,
            &["saved", "reviewed", "shortlisted", "archived"],
            "vacancy status",
        )?;
        let source = input.source_url.trim();
        limited(source, "Source URL", 4000)?;
        let source = if source.is_empty() {
            String::new()
        } else {
            let mut url = url::Url::parse(source)
                .map_err(|_| "Enter a valid http or https source URL.".to_string())?;
            if !["http", "https"].contains(&url.scheme())
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err("Source URL must use http or https without credentials.".into());
            }
            url.set_fragment(None);
            url.to_string()
        };
        let existing: Option<String> = self
            .db
            .query_row(
                "SELECT id FROM vacancies WHERE source_url=?1 AND source_url!=''",
                [&source],
                |r| r.get(0),
            )
            .optional()
            .map_err(error)?;
        if existing.is_some() && existing != input.id {
            return Err("This source URL is already saved. Open the existing vacancy.".into());
        }
        let time = now();
        if let Some(vacancy_id) = input.id {
            let changed = self.db.execute("UPDATE vacancies SET title=?1,company=?2,location=?3,work_mode=?4,source_url=?5,description=?6,notes=?7,status=?8,updated_at=?9 WHERE id=?10", params![title,company,input.location,input.work_mode,source,input.description,input.notes,input.status,time,vacancy_id]).map_err(error)?;
            if changed == 0 {
                return Err("Vacancy no longer exists.".into());
            }
            Ok(vacancy_id)
        } else {
            let vacancy_id = id();
            self.db
                .execute(
                    "INSERT INTO vacancies VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
                    params![
                        vacancy_id,
                        title,
                        company,
                        input.location,
                        input.work_mode,
                        source,
                        input.description,
                        input.notes,
                        input.status,
                        time
                    ],
                )
                .map_err(error)?;
            Ok(vacancy_id)
        }
    }

    fn documents(&self) -> Result<Vec<Document>> {
        let mut stmt = self
            .db
            .prepare("SELECT id,title,kind FROM documents ORDER BY title,id")
            .map_err(error)?;
        let mut docs = stmt
            .query_map([], |r| {
                Ok(Document {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    kind: r.get(2)?,
                    versions: vec![],
                })
            })
            .map_err(error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(error)?;
        let mut versions = self.db.prepare("SELECT id,number,file_name,created_at FROM document_versions WHERE document_id=?1 ORDER BY number DESC").map_err(error)?;
        for doc in &mut docs {
            let records = versions
                .query_map([&doc.id], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                })
                .map_err(error)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(error)?;
            for (id, number, file_name, created_at) in records {
                if file_name != format!("{id}.txt") || Uuid::parse_str(&id).is_err() {
                    return Err("Invalid document file reference.".into());
                }
                let content = fs::read_to_string(self.documents_path.join(file_name)).map_err(|_| format!("Document version {number} of '{}' could not be read. Restore its file from your backup.", doc.title))?;
                doc.versions.push(DocumentVersion {
                    id,
                    number,
                    content,
                    created_at,
                });
            }
        }
        Ok(docs)
    }

    pub fn save_document(&mut self, input: DocumentInput) -> Result<String> {
        let title = required(&input.title, "Document title", 300)?;
        required(&input.content, "Document content", 1_000_000)?;
        one_of(
            &input.kind,
            &["resume", "cover-letter", "note", "job-offer", "agreement", "tax-related", "other"],
            "document type",
        )?;
        let document_id = input.id.clone().unwrap_or_else(id);
        let version_id = id();
        let file_name = format!("{version_id}.txt");
        let file_path = self.documents_path.join(&file_name);
        let tx = self.db.transaction().map_err(error)?;
        if input.id.is_some() {
            let kind: Option<String> = tx
                .query_row(
                    "SELECT kind FROM documents WHERE id=?1",
                    [&document_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(error)?;
            if kind.as_deref() != Some(&input.kind) {
                return Err("Document does not exist or its type has changed.".into());
            }
            tx.execute(
                "UPDATE documents SET title=?1 WHERE id=?2",
                params![title, document_id],
            )
            .map_err(error)?;
        } else {
            tx.execute(
                "INSERT INTO documents VALUES (?1,?2,?3)",
                params![document_id, title, input.kind],
            )
            .map_err(error)?;
        }
        let number: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(number),0)+1 FROM document_versions WHERE document_id=?1",
                [&document_id],
                |r| r.get(0),
            )
            .map_err(error)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .map_err(error)?;
        use std::io::Write;
        if let Err(e) = file
            .write_all(input.content.as_bytes())
            .and_then(|_| file.sync_all())
        {
            drop(file);
            let _ = fs::remove_file(&file_path);
            return Err(error(e));
        }
        drop(file);
        let result = tx
            .execute(
                "INSERT INTO document_versions VALUES (?1,?2,?3,?4,?5)",
                params![version_id, document_id, number, file_name, now()],
            )
            .map_err(error)
            .and_then(|_| tx.commit().map_err(error));
        if result.is_err() {
            let _ = fs::remove_file(&file_path);
        }
        result.map(|_| document_id)
    }

    fn applications(&self) -> Result<Vec<Application>> {
        let mut stmt = self.db.prepare("SELECT id,vacancy_id,stage,next_action,due_date,notes,submitted_at,created_at,updated_at FROM applications ORDER BY updated_at DESC,id").map_err(error)?;
        let mut apps = stmt
            .query_map([], |r| {
                Ok(Application {
                    id: r.get(0)?,
                    vacancy_id: r.get(1)?,
                    stage: r.get(2)?,
                    next_action: r.get(3)?,
                    due_date: r.get(4)?,
                    notes: r.get(5)?,
                    submitted_at: r.get(6)?,
                    created_at: r.get(7)?,
                    updated_at: r.get(8)?,
                    document_version_ids: vec![],
                    events: vec![],
                })
            })
            .map_err(error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(error)?;
        for app in &mut apps {
            let mut docs = self.db.prepare("SELECT version_id FROM application_documents WHERE application_id=?1 ORDER BY version_id").map_err(error)?;
            app.document_version_ids = docs
                .query_map([&app.id], |r| r.get(0))
                .map_err(error)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(error)?;
            let mut events = self.db.prepare("SELECT id,stage,created_at FROM application_events WHERE application_id=?1 ORDER BY rowid DESC").map_err(error)?;
            app.events = events
                .query_map([&app.id], |r| {
                    Ok(ApplicationEvent {
                        id: r.get(0)?,
                        stage: r.get(1)?,
                        created_at: r.get(2)?,
                    })
                })
                .map_err(error)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(error)?;
        }
        Ok(apps)
    }

    pub fn prepare_application(&mut self, vacancy_id: &str) -> Result<String> {
        let tx = self.db.transaction().map_err(error)?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM applications WHERE vacancy_id=?1",
                [vacancy_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(error)?;
        if let Some(existing) = existing {
            return Ok(existing);
        }
        let status: Option<String> = tx
            .query_row(
                "SELECT status FROM vacancies WHERE id=?1",
                [vacancy_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(error)?;
        if status.is_none() || status.as_deref() == Some("archived") {
            return Err("Choose an active vacancy to prepare an application.".into());
        }
        let app_id = id();
        let time = now();
        tx.execute("INSERT INTO applications VALUES (?1,?2,'preparing','Prepare resume and cover letter','','',NULL,?3,?3)",params![app_id,vacancy_id,time]).map_err(error)?;
        tx.execute(
            "INSERT INTO application_events VALUES (?1,?2,'preparing',?3)",
            params![id(), app_id, time],
        )
        .map_err(error)?;
        tx.commit().map_err(error)?;
        Ok(app_id)
    }

    pub fn save_application(&mut self, mut input: ApplicationInput) -> Result<()> {
        one_of(
            &input.stage,
            &[
                "preparing",
                "applied",
                "interview",
                "offer",
                "rejected",
                "withdrawn",
            ],
            "application stage",
        )?;
        limited(&input.next_action, "Next action", 1000)?;
        limited(&input.notes, "Notes", 100_000)?;
        if !input.due_date.is_empty() {
            let date = NaiveDate::parse_from_str(&input.due_date, "%Y-%m-%d")
                .map_err(|_| "Enter a valid due date.".to_string())?;
            if date.format("%Y-%m-%d").to_string() != input.due_date {
                return Err("Enter a valid due date.".into());
            }
            required(&input.next_action, "Next action", 1000)?;
        }
        input.document_version_ids.sort();
        input.document_version_ids.dedup();
        let current = self
            .applications()?
            .into_iter()
            .find(|a| a.id == input.id)
            .ok_or("Application no longer exists.")?;
        if current.submitted_at.is_some()
            && input.document_version_ids != current.document_version_ids
        {
            return Err("Submitted document versions are locked. Create a new document version for future applications.".into());
        }
        if current.submitted_at.is_some() && input.stage == "preparing" {
            return Err("A submitted application cannot return to preparation.".into());
        }
        let submitting = current.submitted_at.is_none()
            && ["applied", "interview", "offer"].contains(&input.stage.as_str());
        if submitting && input.document_version_ids.is_empty() {
            return Err(
                "Attach the document versions you sent before recording submission.".into(),
            );
        }
        let time = now();
        let tx = self.db.transaction().map_err(error)?;
        for version_id in &input.document_version_ids {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM document_versions WHERE id=?1)",
                    [version_id],
                    |r| r.get(0),
                )
                .map_err(error)?;
            if !exists {
                return Err("An attached document version no longer exists.".into());
            }
        }
        tx.execute(
            "DELETE FROM application_documents WHERE application_id=?1",
            [&input.id],
        )
        .map_err(error)?;
        for version_id in &input.document_version_ids {
            tx.execute(
                "INSERT INTO application_documents VALUES (?1,?2)",
                params![input.id, version_id],
            )
            .map_err(error)?;
        }
        let submitted_at = if submitting {
            Some(time.clone())
        } else {
            current.submitted_at
        };
        tx.execute("UPDATE applications SET stage=?1,next_action=?2,due_date=?3,notes=?4,submitted_at=?5,updated_at=?6 WHERE id=?7",params![input.stage,input.next_action,input.due_date,input.notes,submitted_at,time,input.id]).map_err(error)?;
        if current.stage != input.stage {
            tx.execute(
                "INSERT INTO application_events VALUES (?1,?2,?3,?4)",
                params![id(), input.id, input.stage, time],
            )
            .map_err(error)?;
        }
        tx.commit().map_err(error)
    }

    fn profile(&self) -> Result<Profile> {
        let value: Option<String> = self
            .db
            .query_row("SELECT value FROM settings WHERE key='profile'", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(error)?;
        value
            .map(|v| serde_json::from_str(&v).map_err(error))
            .unwrap_or_else(|| Ok(Profile::default()))
    }

    pub fn save_profile(&mut self, profile: Profile) -> Result<()> {
        limited(&profile.name, "Name", 300)?;
        limited(&profile.headline, "Headline", 500)?;
        limited(&profile.email, "Email", 500)?;
        limited(&profile.summary, "Summary", 100_000)?;
        self.db.execute("INSERT INTO settings VALUES ('profile',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[serde_json::to_string(&profile).map_err(error)?]).map_err(error)?;
        Ok(())
    }
}
