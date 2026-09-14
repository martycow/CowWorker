use crate::{DocumentInput, Store};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub base_revision: i64,
    pub base_version: Option<String>,
    pub fingerprint: Option<String>,
    pub fields: Value,
    pub source_id: Option<String>,
    pub operation_id: Option<String>,
    pub status: String,
    pub created_at: String,
}

fn column(field: &str) -> Result<&'static str> {
    match field {
        "title" => Ok("title"),
        "company" => Ok("company"),
        "location" => Ok("location"),
        "workMode" => Ok("work_mode"),
        "sourceUrl" => Ok("source_url"),
        "description" => Ok("description"),
        "notes" => Ok("notes"),
        "status" => Ok("status"),
        "structured" => Ok("structured"),
        _ => Err(format!("Unsupported vacancy field: {field}")),
    }
}
pub(crate) fn validate(field: &str, value: &Value) -> Result<String> {
    if field == "structured" {
        if value.to_string().len() > 100_000 {
            return Err("Structured fields exceed 100000 bytes.".into());
        }
        let object = value
            .as_object()
            .ok_or("Structured fields must be an object.")?;
        for (key, value) in object {
            match key.as_str() {
                "salaryMin" | "salaryMax" => {
                    if !value.is_null()
                        && !value.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0)
                    {
                        return Err("Salary must be a nonnegative number or unknown.".into());
                    }
                }
                "requirements" | "responsibilities" | "stack" | "benefits" => {
                    if !value.is_null()
                        && !value.as_array().is_some_and(|a| {
                            a.len() <= 1000
                                && a.iter()
                                    .all(|v| v.as_str().is_some_and(|s| s.len() <= 4000))
                        })
                    {
                        return Err("Expected a bounded list of text values.".into());
                    }
                }
                "salaryRaw" | "currency" | "salaryPeriod" | "employmentType" | "postedAt"
                | "closingAt" | "sourceName" => {
                    if !value.is_null() && !value.as_str().is_some_and(|s| s.len() <= 4000) {
                        return Err("Expected text or unknown.".into());
                    }
                }
                _ => return Err(format!("Unsupported structured field: {key}")),
            }
        }
        if let (Some(min), Some(max)) = (value["salaryMin"].as_f64(), value["salaryMax"].as_f64()) {
            if min > max {
                return Err("Salary minimum exceeds maximum.".into());
            }
        }
        return serde_json::to_string(value).map_err(err);
    }
    let text = value.as_str().ok_or("Expected a text field value.")?;
    let max = match field {
        "title" | "company" => 300,
        "location" => 500,
        "description" => 200_000,
        "notes" => 100_000,
        "sourceUrl" => 4000,
        _ => 100,
    };
    if text.len() > max {
        return Err(format!("{field} exceeds {max} bytes."));
    }
    if ["title", "company"].contains(&field) && text.trim().is_empty() {
        return Err(format!("{field} is required."));
    }
    if field == "workMode" && !["Remote", "Hybrid", "On-site", "Unspecified"].contains(&text) {
        return Err("Invalid work mode.".into());
    }
    if field == "status" && !["saved", "reviewed", "shortlisted", "archived"].contains(&text) {
        return Err("Invalid vacancy status.".into());
    }
    if field == "sourceUrl" && !text.trim().is_empty() {
        let mut url = url::Url::parse(text.trim()).map_err(err)?;
        if !["http", "https"].contains(&url.scheme())
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("Source URL must use http or https without credentials.".into());
        }
        url.set_fragment(None);
        return Ok(url.to_string());
    }
    Ok(if ["title", "company", "sourceUrl"].contains(&field) {
        text.trim().into()
    } else {
        text.into()
    })
}
fn revision(db: &Connection, entity: &str, expected: i64) -> Result<()> {
    let current: Option<i64> = db
        .query_row(
            "SELECT revision FROM vacancies WHERE id=?1",
            [entity],
            |r| r.get(0),
        )
        .optional()
        .map_err(err)?;
    if current != Some(expected) {
        return Err(format!("Revision conflict: expected {expected}, current {current:?}. Reload and review your draft."));
    }
    Ok(())
}
fn write_fields(
    db: &Connection,
    entity: &str,
    fields: &Map<String, Value>,
    next: i64,
    manual: bool,
) -> Result<()> {
    for (field, value) in fields {
        let col = column(field)?;
        let text = validate(field, value)?;
        db.execute(
            &format!("UPDATE vacancies SET {col}=?1 WHERE id=?2"),
            params![text, entity],
        )
        .map_err(err)?;
        if field == "company" {
            db.execute("INSERT INTO companies(id,name,provisional,created_at,updated_at) SELECT ?1,?2,1,?3,?3 WHERE NOT EXISTS(SELECT 1 FROM companies WHERE name=?2 AND provisional=1 AND merged_into IS NULL)",params![uuid::Uuid::new_v4().to_string(),text,chrono::Utc::now().to_rfc3339()]).map_err(err)?;
            db.execute("UPDATE vacancies SET company_id=(SELECT id FROM companies WHERE name=?1 AND provisional=1 AND merged_into IS NULL) WHERE id=?2",params![text,entity]).map_err(err)?;
        }
        if manual {
            let effective = if field == "structured" {
                serde_json::from_str::<Value>(&text).map_err(err)?
            } else {
                Value::String(text)
            };
            db.execute("INSERT INTO field_overrides VALUES('vacancy',?1,?2,?3,?4) ON CONFLICT(entity_type,entity_id,field) DO UPDATE SET value=excluded.value,revision=excluded.revision",params![entity,field,effective.to_string(),next]).map_err(err)?;
        }
    }
    db.execute(
        "UPDATE vacancies SET revision=?1,updated_at=?2 WHERE id=?3",
        params![next, chrono::Utc::now().to_rfc3339(), entity],
    )
    .map_err(err)?;
    Ok(())
}
impl Store {
    pub fn patch_vacancy(
        &mut self,
        entity: &str,
        expected: i64,
        fields: Map<String, Value>,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        revision(&tx, entity, expected)?;
        if !fields.is_empty() {
            write_fields(&tx, entity, &fields, expected + 1, true)?;
        }
        tx.commit().map_err(err)
    }
    pub fn proposals(&self, entity: &str) -> Result<Vec<Proposal>> {
        let mut stmt=self.db.prepare("SELECT id,entity_type,entity_id,base_revision,base_version,fingerprint,fields,source_id,operation_id,status,created_at FROM change_proposals WHERE entity_id=?1 ORDER BY created_at DESC,id LIMIT 100").map_err(err)?;
        let rows = stmt
            .query_map([entity], |r| {
                Ok(Proposal {
                    id: r.get(0)?,
                    entity_type: r.get(1)?,
                    entity_id: r.get(2)?,
                    base_revision: r.get(3)?,
                    base_version: r.get(4)?,
                    fingerprint: r.get(5)?,
                    fields: serde_json::from_str(&r.get::<_, String>(6)?).unwrap_or_default(),
                    source_id: r.get(7)?,
                    operation_id: r.get(8)?,
                    status: r.get(9)?,
                    created_at: r.get(10)?,
                })
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(rows)
    }
    pub fn propose_vacancy(
        &mut self,
        entity: &str,
        expected: i64,
        fields: Map<String, Value>,
        source: Option<&str>,
        operation: Option<&str>,
    ) -> Result<String> {
        if fields.is_empty() {
            return Err("A proposal needs at least one field.".into());
        }
        for (k, v) in &fields {
            column(k)?;
            validate(k, v)?;
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        // Late results stay reviewable at their original revision.
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM vacancies WHERE id=?1)",
                [entity],
                |r| r.get(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("Vacancy no longer exists.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let time = chrono::Utc::now().to_rfc3339();
        for (k, v) in &fields {
            tx.execute("INSERT INTO field_candidates(id,entity_type,entity_id,field,value,raw_value,source_id,operation_id,created_at) VALUES(?1,'vacancy',?2,?3,?4,?4,?5,?6,?7)",params![uuid::Uuid::new_v4().to_string(),entity,k,v.to_string(),source,operation,time]).map_err(err)?;
        }
        tx.execute("INSERT INTO change_proposals(id,entity_type,entity_id,base_revision,fields,source_id,operation_id,created_at) VALUES(?1,'vacancy',?2,?3,?4,?5,?6,?7)",params![id,entity,expected,Value::Object(fields).to_string(),source,operation,time]).map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(id)
    }
    pub fn review_proposal(
        &mut self,
        proposal_id: &str,
        accept: bool,
        replace_overrides: bool,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let (entity,kind,expected,fields,status):(String,String,i64,String,String)=tx.query_row("SELECT entity_id,entity_type,base_revision,fields,status FROM change_proposals WHERE id=?1",[proposal_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(err)?;
        if status != "review" {
            return Err("This proposal has already been reviewed.".into());
        }
        if accept && kind == "company" {
            let (current, stored): (i64, String) = tx
                .query_row(
                    "SELECT revision,fields FROM companies WHERE id=?1 AND merged_into IS NULL",
                    [&entity],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(err)?;
            if current != expected {
                return Err(
                    "Revision conflict. Reload the company and review the newer values.".into(),
                );
            }
            let proposed: Value = serde_json::from_str(&fields).map_err(err)?;
            crate::companies::validate_fields(&proposed)?;
            let mut values: Value = serde_json::from_str(&stored).map_err(err)?;
            for (field, value) in proposed.as_object().ok_or("Invalid company proposal")? {
                let protected:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM field_overrides WHERE entity_type='company' AND entity_id=?1 AND field=?2)",params![entity,field],|r|r.get(0)).map_err(err)?;
                if protected && !replace_overrides {
                    return Err(format!(
                        "{field} has a user override. Explicit replacement is required."
                    ));
                }
                values[field] = value.clone();
                if replace_overrides {
                    tx.execute("INSERT INTO field_overrides VALUES('company',?1,?2,?3,?4) ON CONFLICT(entity_type,entity_id,field) DO UPDATE SET value=excluded.value,revision=excluded.revision",params![entity,field,value.to_string(),expected+1]).map_err(err)?;
                }
                tx.execute("INSERT INTO field_candidates(id,entity_type,entity_id,field,value,raw_value,source_id,accepted,created_at) SELECT ?1,'company',?2,?3,?4,?4,source_id,1,?5 FROM change_proposals WHERE id=?6",params![uuid::Uuid::new_v4().to_string(),entity,field,value.to_string(),chrono::Utc::now().to_rfc3339(),proposal_id]).map_err(err)?;
            }
            tx.execute(
                "UPDATE companies SET fields=?1,revision=revision+1,updated_at=?2 WHERE id=?3",
                params![values.to_string(), chrono::Utc::now().to_rfc3339(), entity],
            )
            .map_err(err)?;
        } else if accept {
            if kind != "vacancy" {
                return Err("Use document version review for this proposal.".into());
            }
            revision(&tx, &entity, expected)?;
            let fields: Map<String, Value> = serde_json::from_str(&fields).map_err(err)?;
            for (field, value) in &fields {
                let protected:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM field_overrides WHERE entity_type='vacancy' AND entity_id=?1 AND field=?2)",params![entity,field],|r|r.get(0)).map_err(err)?;
                if protected && !replace_overrides {
                    return Err(format!("{field} has a user override. Explicitly choose Replace user values to apply."));
                }
                tx.execute("UPDATE field_candidates SET accepted=0 WHERE entity_type='vacancy' AND entity_id=?1 AND field=?2",params![entity,field]).map_err(err)?;
                tx.execute("INSERT INTO field_candidates(id,entity_type,entity_id,field,value,raw_value,accepted,created_at) VALUES(?1,'vacancy',?2,?3,?4,?4,1,?5)",params![uuid::Uuid::new_v4().to_string(),entity,field,value.to_string(),chrono::Utc::now().to_rfc3339()]).map_err(err)?;
            }
            write_fields(&tx, &entity, &fields, expected + 1, replace_overrides)?;
        }
        tx.execute(
            "UPDATE change_proposals SET status=?1 WHERE id=?2",
            params![if accept { "accepted" } else { "dismissed" }, proposal_id],
        )
        .map_err(err)?;
        tx.commit().map_err(err)
    }
    pub fn clear_override(&mut self, entity: &str, expected: i64, field: &str) -> Result<()> {
        column(field)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        revision(&tx, entity, expected)?;
        let candidate:Option<String>=tx.query_row("SELECT value FROM field_candidates WHERE entity_type='vacancy' AND entity_id=?1 AND field=?2 AND accepted=1 ORDER BY created_at DESC,id DESC LIMIT 1",params![entity,field],|r|r.get(0)).optional().map_err(err)?;
        let value: Value = serde_json::from_str(
            &candidate
                .ok_or("No accepted source value is available. Your override was retained.")?,
        )
        .map_err(err)?;
        tx.execute(
            "DELETE FROM field_overrides WHERE entity_type='vacancy' AND entity_id=?1 AND field=?2",
            params![entity, field],
        )
        .map_err(err)?;
        write_fields(
            &tx,
            entity,
            &Map::from_iter([(field.into(), value)]),
            expected + 1,
            false,
        )?;
        tx.commit().map_err(err)
    }
    pub fn document_content(&self, version_id: &str) -> Result<String> {
        let file: String = self
            .db
            .query_row(
                "SELECT file_name FROM document_versions WHERE id=?1",
                [version_id],
                |r| r.get(0),
            )
            .map_err(err)?;
        if uuid::Uuid::parse_str(version_id).is_err() || file != format!("{version_id}.txt") {
            return Err("Invalid document file reference.".into());
        }
        std::fs::read_to_string(self.documents_path.join(file)).map_err(|_| {
            "Document version could not be read. Restore its file from your backup.".into()
        })
    }
    pub fn correct_document(
        &mut self,
        entity: &str,
        expected: i64,
        title: &str,
        kind: &str,
    ) -> Result<()> {
        if title.trim().is_empty() || title.len() > 300 {
            return Err("Document title must contain 1 to 300 bytes.".into());
        }
        let changed=self.db.execute("UPDATE documents SET title=?1,kind=?2,revision=revision+1 WHERE id=?3 AND revision=?4",params![title.trim(),kind,entity,expected]).map_err(err)?;
        if changed != 1 {
            return Err("Revision conflict. Reload the document metadata.".into());
        }
        Ok(())
    }
    pub fn propose_document(
        &mut self,
        entity: &str,
        expected: i64,
        base_version: &str,
        content: &str,
    ) -> Result<String> {
        self.propose_document_operation(entity, expected, base_version, content, None)
    }
    pub(crate) fn propose_document_operation(
        &mut self,
        entity: &str,
        expected: i64,
        base_version: &str,
        content: &str,
        operation: Option<&str>,
    ) -> Result<String> {
        if content.is_empty() || content.len() > 1_000_000 {
            return Err("Proposal content must contain 1 to 1000000 bytes.".into());
        }
        let belongs: bool = self
            .db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM document_versions WHERE id=?1 AND document_id=?2)",
                params![base_version, entity],
                |r| r.get(0),
            )
            .map_err(err)?;
        if !belongs {
            return Err("Version does not belong to the target document.".into());
        }
        let original = self.document_content(base_version)?;
        let fingerprint = format!("{:x}", Sha256::digest(original.as_bytes()));
        let id = uuid::Uuid::new_v4().to_string();
        self.db.execute("INSERT INTO change_proposals(id,entity_type,entity_id,base_revision,base_version,fingerprint,fields,created_at,operation_id) VALUES(?1,'document',?2,?3,?4,?5,?6,?7,?8)",params![id,entity,expected,base_version,fingerprint,serde_json::json!({"content":content}).to_string(),chrono::Utc::now().to_rfc3339(),operation]).map_err(err)?;
        Ok(id)
    }
    pub fn accept_document_proposal(&mut self, proposal_id: &str) -> Result<String> {
        // The revision guard and proposal status commit with the new immutable version.
        let (entity,expected,base,fingerprint,fields,status):(String,i64,String,String,String,String)=self.db.query_row("SELECT entity_id,base_revision,base_version,fingerprint,fields,status FROM change_proposals WHERE id=?1 AND entity_type='document'",[proposal_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(err)?;
        if status != "review" {
            return Err("This proposal has already been reviewed.".into());
        }
        let current = self.document_content(&base)?;
        if format!("{:x}", Sha256::digest(current.as_bytes())) != fingerprint {
            return Err("Document fingerprint changed. Review a new proposal.".into());
        }
        let (title, kind): (String, String) = self
            .db
            .query_row(
                "SELECT title,kind FROM documents WHERE id=?1",
                [&entity],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(err)?;
        let fields: Value = serde_json::from_str(&fields).map_err(err)?;
        let id = self.save_document_review(
            DocumentInput {
                id: Some(entity),
                expected_revision: Some(expected),
                title,
                kind,
                content: fields["content"]
                    .as_str()
                    .ok_or("Invalid proposal content")?
                    .into(),
            },
            Some(proposal_id),
        )?;
        Ok(id)
    }
}
