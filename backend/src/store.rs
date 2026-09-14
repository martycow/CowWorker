use crate::model::*;
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::{fs, path::PathBuf, time::Duration};
use uuid::Uuid;

type Result<T> = std::result::Result<T, String>;
pub struct Store {
    pub(crate) db: Connection,
    pub(crate) documents_path: PathBuf,
    pub(crate) path: PathBuf,
    pub(crate) extractor: Option<PathBuf>,
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
        if version > crate::migrations::CURRENT_VERSION {
            return Err("This workspace was created by a newer CowWorker version. Update CowWorker before opening it.".into());
        }
        let before_upgrade = if version > 0 && version < crate::migrations::CURRENT_VERSION {
            Some(crate::backup::create(
                &db,
                &path,
                &path
                    .join("backups")
                    .join(format!("before-schema-{version}-{}", id())),
            )?)
        } else {
            None
        };
        crate::migrations::run(&db)?;
        if let Some(snapshot) = before_upgrade {
            for table in crate::migrations::PRESERVED_TABLES {
                let count: i64 = db
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                    .map_err(error)?;
                if snapshot.counts.get(table) != Some(&count) {
                    return Err(format!("Upgrade record check failed for {table}. Open the verified before-schema backup under {}. The application will not use this workspace.",path.join("backups").display()));
                }
            }
        }
        Ok(Self {
            db,
            documents_path,
            path,
            extractor: None,
        })
    }

    pub fn use_extractor(&mut self, executable: PathBuf) {
        self.extractor = Some(executable);
    }

    pub fn workspace(&self) -> Result<Workspace> {
        Ok(Workspace {
            schema_version: crate::migrations::CURRENT_VERSION,
            vacancies: self.vacancies()?,
            documents: self.documents()?,
            applications: self.applications()?,
            profile: self.profile()?,
        })
    }

    pub(crate) fn vacancies(&self) -> Result<Vec<Vacancy>> {
        let mut stmt = self.db.prepare("SELECT id,title,company,location,work_mode,source_url,description,notes,status,created_at,updated_at,revision,structured,company_id FROM vacancies ORDER BY created_at DESC,id").map_err(error)?;
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
                    revision: r.get(11)?,
                    structured: serde_json::from_str(&r.get::<_, String>(12)?).unwrap_or_default(),
                    company_id: r.get(13)?,
                })
            })
            .map_err(error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(error);
        records
    }

    pub fn save_vacancy(&mut self, input: VacancyInput) -> Result<String> {
        if let Some(ref vacancy_id) = input.id {
            let current = self
                .vacancies()?
                .into_iter()
                .find(|v| &v.id == vacancy_id)
                .ok_or("Vacancy no longer exists.")?;
            let expected = input
                .expected_revision
                .ok_or("Reload the vacancy before editing: expected revision is required.")?;
            let old = serde_json::to_value(&current).map_err(error)?;
            let new = serde_json::to_value(&input).map_err(error)?;
            let fields = new
                .as_object()
                .ok_or("Invalid vacancy")?
                .iter()
                .filter(|(k, v)| {
                    !["id", "expectedRevision"].contains(&k.as_str()) && old.get(*k) != Some(*v)
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            self.patch_vacancy(vacancy_id, expected, fields)?;
            return Ok(vacancy_id.clone());
        }
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
        let vacancy_id = id();
        self.db
                .execute(
                    "INSERT INTO vacancies(id,title,company,location,work_mode,source_url,description,notes,status,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
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

    fn documents(&self) -> Result<Vec<Document>> {
        let mut stmt = self
            .db
            .prepare("SELECT id,title,kind,revision FROM documents ORDER BY title,id")
            .map_err(error)?;
        let mut docs = stmt
            .query_map([], |r| {
                Ok(Document {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    kind: r.get(2)?,
                    revision: r.get(3)?,
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
                let content = None;
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
        self.save_document_review(input, None)
    }

    pub(crate) fn save_document_review(
        &mut self,
        input: DocumentInput,
        proposal: Option<&str>,
    ) -> Result<String> {
        let title = required(&input.title, "Document title", 300)?;
        required(&input.content, "Document content", 1_000_000)?;
        one_of(
            &input.kind,
            &[
                "resume",
                "cover-letter",
                "note",
                "job-offer",
                "agreement",
                "tax-related",
                "other",
                "job-description",
                "portfolio",
                "reference",
                "unknown",
            ],
            "document type",
        )?;
        let document_id = input.id.clone().unwrap_or_else(id);
        let version_id = id();
        let file_name = format!("{version_id}.txt");
        let file_path = self.documents_path.join(&file_name);
        let tx = self.db.savepoint().map_err(error)?;
        if let Some(proposal) = proposal {
            let matches:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM change_proposals p WHERE p.id=?1 AND p.base_version=(SELECT id FROM document_versions WHERE document_id=p.entity_id ORDER BY number DESC LIMIT 1))",[proposal],|r|r.get(0)).map_err(error)?;
            if !matches {
                return Err("The proposal targets an older document version. Generate a fresh proposal for the latest version.".into());
            }
            let changed = tx
                .execute(
                    "UPDATE change_proposals SET status='accepted' WHERE id=?1 AND status='review'",
                    [proposal],
                )
                .map_err(error)?;
            if changed != 1 {
                return Err("This proposal has already been reviewed.".into());
            }
        }
        if input.id.is_some() {
            let expected = input
                .expected_revision
                .ok_or("Reload the document before editing: expected revision is required.")?;
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
                "UPDATE documents SET title=?1,revision=revision+1 WHERE id=?2 AND revision=?3",
                params![title, document_id, expected],
            )
            .map_err(error)
            .and_then(|count| {
                if count == 1 {
                    Ok(())
                } else {
                    Err(
                        "Revision conflict. Reload the document; your draft remains available."
                            .into(),
                    )
                }
            })?;
        } else {
            tx.execute(
                "INSERT INTO documents(id,title,kind) VALUES (?1,?2,?3)",
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
        let tx = self.db.savepoint().map_err(error)?;
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
        let tx = self.db.savepoint().map_err(error)?;
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
