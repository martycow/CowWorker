use crate::{DocumentInput, Store};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub mod formats;
pub mod html;
pub mod ocr;
pub mod pdf;
pub mod process;
pub mod web;
type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportDraft {
    pub entity_type: String,
    pub kind: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub work_mode: String,
    pub source_url: String,
    pub content: String,
    pub structured: Value,
    pub confidence: f64,
    pub method: String,
}
pub fn classify(text: &str, name: &str) -> ImportDraft {
    let lower = text.to_lowercase();
    let job = lower.starts_with("job posting\n")
        || (lower.contains("requirements")
            && (lower.contains("responsibilities") || lower.contains("salary")))
        || (lower.contains("требования")
            && (lower.contains("обязанности") || lower.contains("зарплата")));
    let resume = (lower.contains("experience") && lower.contains("education"))
        || (lower.contains("опыт") && lower.contains("образование"));
    let mut draft = ImportDraft {
        entity_type: if job { "vacancy" } else { "document" }.into(),
        kind: if job {
            "job-description"
        } else if resume {
            "resume"
        } else {
            "unknown"
        }
        .into(),
        title: name
            .trim_end_matches(".txt")
            .trim_end_matches(".md")
            .trim_end_matches(".pdf")
            .trim_end_matches(".docx")
            .chars()
            .take(200)
            .collect(),
        company: String::new(),
        location: String::new(),
        work_mode: "Unspecified".into(),
        source_url: String::new(),
        content: text.into(),
        structured: json!({}),
        confidence: if job || resume { 0.7 } else { 0.0 },
        method: "keyword rules v1; review required".into(),
    };
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim();
            match key.trim().to_lowercase().as_str() {
                "company" | "employer" | "компания" | "работодатель" => {
                    draft.company = value.into()
                }
                "title" | "role" | "должность" | "вакансия" => {
                    draft.title = value.into()
                }
                "location" | "город" | "местоположение" => {
                    draft.location = value.into()
                }
                "salary" | "зарплата" => draft.structured["salaryRaw"] = value.into(),
                "employment type" => draft.structured["employmentType"] = value.into(),
                "work mode" if ["Remote", "Hybrid", "On-site", "Unspecified"].contains(&value) => {
                    draft.work_mode = value.into()
                }
                _ => (),
            }
        }
    }
    draft
}
impl Store {
    pub fn import_bytes(
        &mut self,
        name: &str,
        media_type: &str,
        bytes: &[u8],
        url: Option<&str>,
    ) -> Result<String> {
        let source = self.acquire_source(name, media_type, bytes, url)?;
        let item = uuid::Uuid::new_v4().to_string();
        let session = uuid::Uuid::new_v4().to_string();
        self.db.execute_batch("BEGIN IMMEDIATE").map_err(err)?;
        let result = (|| {
            let task = self.enqueue_task(
                "extract-text",
                json!({"sourceId":source.id,"mediaType":media_type,"name":name,"sourceUrl":url}),
                &format!("import:{item}"),
                None,
            )?;
            self.db
                .execute(
                    "INSERT INTO import_sessions VALUES(?1,?2)",
                    params![session, chrono::Utc::now().timestamp()],
                )
                .map_err(err)?;
            self.db.execute("INSERT INTO import_items(id,session_id,source_id,task_id,created_at) VALUES(?1,?2,?3,?4,?5)",params![item,session,source.id,task,chrono::Utc::now().timestamp()]).map_err(err)?;
            self.db
                .execute(
                    "INSERT INTO import_item_sources VALUES(?1,?2,0)",
                    params![item, source.id],
                )
                .map_err(err)?;
            self.db.execute("INSERT INTO stage_runs VALUES(?1,?2,'acquisition','managed-copy','1','completed',?3,?4)",params![uuid::Uuid::new_v4().to_string(),item,source.sha256,chrono::Utc::now().timestamp()]).map_err(err)?;
            Ok(item)
        })();
        match result {
            Ok(id) => {
                self.db.execute_batch("COMMIT").map_err(err)?;
                Ok(id)
            }
            Err(e) => {
                let _ = self.db.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }
    pub fn imports(&self, offset: i64) -> Result<Value> {
        let mut stmt=self.db.prepare("SELECT i.id,CASE WHEN (SELECT COUNT(*) FROM import_item_sources x WHERE x.item_id=i.id)>1 THEN 'Combined sources' ELSE s.name END,s.media_type,i.revision,i.status,COALESCE(i.draft,json_extract(t.result,'$.draft')),t.status,t.error,i.target_id,i.target_type,i.source_id FROM import_items i JOIN sources s ON s.id=i.source_id JOIN background_tasks t ON t.id=i.task_id ORDER BY i.created_at DESC,i.rowid DESC LIMIT 25 OFFSET ?1").map_err(err)?;
        let rows=stmt.query_map([offset.max(0)],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"mediaType":r.get::<_,String>(2)?,"revision":r.get::<_,i64>(3)?,"status":r.get::<_,String>(4)?,"draft":r.get::<_,Option<String>>(5)?.and_then(|s|serde_json::from_str::<Value>(&s).ok()),"taskStatus":r.get::<_,String>(6)?,"error":r.get::<_,Option<String>>(7)?,"targetId":r.get::<_,Option<String>>(8)?,"targetType":r.get::<_,Option<String>>(9)?,"sourceId":r.get::<_,String>(10)?}))).map_err(err)?.collect::<std::result::Result<Vec<_>,_>>().map_err(err)?;
        Ok(rows.into())
    }
    pub fn review_import(
        &mut self,
        item: &str,
        expected: i64,
        draft: ImportDraft,
        save: bool,
    ) -> Result<Option<String>> {
        crate::provenance::validate("structured", &draft.structured)?;
        if draft.content.len() > 1_000_000
            || draft.title.trim().is_empty()
            || draft.title.len() > 300
        {
            return Err("Review needs a title and content within the document limits.".into());
        }
        if !["vacancy", "document"].contains(&draft.entity_type.as_str()) {
            return Err("Choose Vacancy or Document.".into());
        }
        self.db.execute_batch("BEGIN IMMEDIATE").map_err(err)?;
        let result = (|| {
            let (revision, source, status, target): (i64, String, String, Option<String>) = self
                .db
                .query_row(
                    "SELECT revision,source_id,status,target_id FROM import_items WHERE id=?1",
                    [item],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .map_err(err)?;
            if status == "saved" {
                return Ok(target);
            }
            if revision != expected {
                return Err("Revision conflict. Reload the import draft.".into());
            }
            let target = if save {
                let target = if draft.entity_type == "document" {
                    self.save_document(DocumentInput {
                        id: None,
                        expected_revision: None,
                        title: draft.title.clone(),
                        kind: draft.kind.clone(),
                        content: draft.content.clone(),
                    })?
                } else {
                    let input=serde_json::from_value(json!({"title":draft.title,"company":draft.company,"location":draft.location,"workMode":draft.work_mode,"sourceUrl":draft.source_url,"description":draft.content,"notes":"","status":"saved"})).map_err(err)?;
                    let id = self.save_vacancy(input)?;
                    // Import review is an explicit user decision; the saved values become user-owned.
                    self.db
                        .execute(
                            "UPDATE vacancies SET structured=?1 WHERE id=?2",
                            params![draft.structured.to_string(), id],
                        )
                        .map_err(err)?;
                    self.db.execute("INSERT INTO field_overrides VALUES('vacancy',?1,'structured',?2,1) ON CONFLICT(entity_type,entity_id,field) DO UPDATE SET value=excluded.value",params![id,draft.structured.to_string()]).map_err(err)?;
                    id
                };
                self.link_source(&draft.entity_type, &target, &source)?;
                self.db.execute("INSERT OR IGNORE INTO entity_sources SELECT ?1,?2,source_id FROM import_item_sources WHERE item_id=?3",params![draft.entity_type,target,item]).map_err(err)?;
                let extracted:Option<String>=self.db.query_row("SELECT json_extract(t.result,'$.sourceId') FROM background_tasks t JOIN import_items i ON i.task_id=t.id WHERE i.id=?1",[item],|r|r.get(0)).map_err(err)?;
                if let Some(extracted) = extracted {
                    self.link_source(&draft.entity_type, &target, &extracted)?;
                }
                Some(target)
            } else {
                None
            };
            self.db.execute("UPDATE import_items SET draft=?1,revision=revision+1,status=?2,target_id=?3,target_type=?4 WHERE id=?5",params![serde_json::to_string(&draft).map_err(err)?,if save{"saved"}else{"review"},target,draft.entity_type,item]).map_err(err)?;
            self.db
                .execute(
                    "INSERT INTO stage_runs VALUES(?1,?2,?3,'user-review','1','completed',NULL,?4)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        item,
                        if save { "save" } else { "review" },
                        chrono::Utc::now().timestamp()
                    ],
                )
                .map_err(err)?;
            Ok(target)
        })();
        match result {
            Ok(target) => {
                self.db.execute_batch("COMMIT").map_err(err)?;
                Ok(target)
            }
            Err(e) => {
                let _ = self.db.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }
    pub fn source_media_type(&self, id: &str) -> Result<String> {
        self.db
            .query_row("SELECT media_type FROM sources WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(err)?
            .ok_or("Source no longer exists.".into())
    }
    pub fn combine_imports(&mut self, items: Vec<String>) -> Result<String> {
        if !(2..=20).contains(&items.len())
            || items.iter().collect::<std::collections::HashSet<_>>().len() != items.len()
        {
            return Err("Select 2 to 20 different imports in reading order.".into());
        }
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(err)?;
        let mut content = String::new();
        let mut sources = Vec::new();
        for item in &items {
            let (status,task_status,draft,source):(String,String,Option<String>,String)=tx.query_row("SELECT i.status,t.status,COALESCE(i.draft,json_extract(t.result,'$.draft')),i.source_id FROM import_items i JOIN background_tasks t ON t.id=i.task_id WHERE i.id=?1",[item],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(err)?;
            if status == "saved" || ["queued", "running"].contains(&task_status.as_str()) {
                return Err("Wait for extraction and select unsaved imports only.".into());
            }
            let draft: ImportDraft = serde_json::from_str(
                &draft.ok_or("Review or transcribe every selected source first.")?,
            )
            .map_err(err)?;
            if draft.content.trim().is_empty() {
                return Err("A selected source has no reviewed text.".into());
            }
            if !content.is_empty() {
                content.push_str("\n\n");
            }
            content.push_str(&draft.content);
            if content.len() > 1_000_000 {
                return Err("Combined text exceeds 1 MB.".into());
            }
            sources.push(source);
            let mut query = tx
                .prepare(
                    "SELECT source_id FROM import_item_sources WHERE item_id=?1 ORDER BY position",
                )
                .map_err(err)?;
            for row in query
                .query_map([item], |r| r.get::<_, String>(0))
                .map_err(err)?
            {
                sources.push(row.map_err(err)?);
            }
            let downloaded:Option<String>=tx.query_row("SELECT json_extract(t.result,'$.sourceId') FROM background_tasks t JOIN import_items i ON i.task_id=t.id WHERE i.id=?1",[item],|r|r.get(0)).map_err(err)?;
            if let Some(source) = downloaded {
                sources.push(source);
            }
        }
        let mut seen = std::collections::HashSet::new();
        sources.retain(|source| seen.insert(source.clone()));
        let item = uuid::Uuid::new_v4().to_string();
        let session = uuid::Uuid::new_v4().to_string();
        let task = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let mut draft = classify(&content, "Combined vacancy");
        draft.entity_type = "vacancy".into();
        draft.kind = "job-description".into();
        draft.method = "Combined sources in selected reading order; review required".into();
        tx.execute(
            "INSERT INTO import_sessions VALUES(?1,?2)",
            params![session, now],
        )
        .map_err(err)?;
        tx.execute("INSERT INTO background_tasks(id,kind,payload,idempotency_key,status,stage,progress,result,created_at,updated_at) VALUES(?1,'extract-text','{}',?2,'completed','Combined sources',100,?3,?4,?4)",params![task,format!("combined:{item}"),json!({"draft":draft}).to_string(),now]).map_err(err)?;
        tx.execute("INSERT INTO import_items(id,session_id,source_id,task_id,draft,status,created_at) VALUES(?1,?2,?3,?4,?5,'review',?6)",params![item,session,sources[0],task,serde_json::to_string(&draft).map_err(err)?,now]).map_err(err)?;
        for (position, source) in sources.iter().enumerate() {
            tx.execute(
                "INSERT INTO import_item_sources VALUES(?1,?2,?3)",
                params![item, source, position as i64],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)?;
        Ok(item)
    }
}
