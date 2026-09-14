use crate::Store;
mod assets;
mod research;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: String,
    pub name: String,
    pub fields: Value,
    pub notes: String,
    pub revision: i64,
    pub provisional: bool,
    pub vacancy_count: i64,
    pub stages: Value,
    pub relationships: Value,
    pub updated_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompanyInput {
    pub id: Option<String>,
    pub expected_revision: Option<i64>,
    pub name: String,
    pub fields: Value,
    pub notes: String,
}
pub(crate) fn validate_fields(fields: &Value) -> Result<()> {
    let object = fields
        .as_object()
        .ok_or("Company fields must be an object.")?;
    for (key, value) in object {
        if ![
            "description",
            "industry",
            "website",
            "headquarters",
            "remotePolicy",
            "founded",
            "employeeCount",
            "stack",
            "contacts",
            "socialLinks",
            "leadership",
        ]
        .contains(&key.as_str())
        {
            return Err(format!("Unsupported company field: {key}"));
        }
        if !value.is_null() && !value.as_str().is_some_and(|s| s.len() <= 10000) {
            return Err("Company facts must be bounded text or unknown.".into());
        }
        if key == "website" {
            if let Some(text) = value.as_str().filter(|s| !s.is_empty()) {
                let url = url::Url::parse(text).map_err(err)?;
                if !["http", "https"].contains(&url.scheme())
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return Err("Enter an HTTP(S) website without credentials.".into());
                }
            }
        }
    }
    Ok(())
}
impl Store {
    pub fn companies(&self, query: &str, offset: i64, limit: i64) -> Result<Vec<Company>> {
        let mut stmt=self.db.prepare("SELECT c.id,c.name,c.fields,c.notes,c.revision,c.provisional,(SELECT COUNT(*) FROM vacancies WHERE company_id=c.id),c.updated_at FROM companies c WHERE c.merged_into IS NULL AND instr(lower(c.name),lower(?1))>0 ORDER BY c.name,c.id LIMIT ?2 OFFSET ?3").map_err(err)?;
        let mut companies = stmt
            .query_map(params![query, limit.clamp(1, 100), offset.max(0)], |r| {
                Ok(Company {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    fields: serde_json::from_str(&r.get::<_, String>(2)?).unwrap_or_default(),
                    notes: r.get(3)?,
                    revision: r.get(4)?,
                    provisional: r.get(5)?,
                    vacancy_count: r.get(6)?,
                    updated_at: r.get(7)?,
                    stages: serde_json::json!({}),
                    relationships: serde_json::json!([]),
                })
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        for company in &mut companies {
            let mut stages=self.db.prepare("SELECT a.stage,COUNT(*) FROM applications a JOIN vacancies v ON v.id=a.vacancy_id WHERE v.company_id=?1 GROUP BY a.stage").map_err(err)?;
            for row in stages
                .query_map([&company.id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
                })
                .map_err(err)?
            {
                let (stage, count) = row.map_err(err)?;
                company.stages[stage] = count.into();
            }
            let mut relationships=self.db.prepare("SELECT id,kind,role,start_date,end_date FROM employment_relationships WHERE company_id=?1 ORDER BY start_date DESC,id").map_err(err)?;
            let rows=relationships.query_map([&company.id],|r|Ok(serde_json::json!({"id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"role":r.get::<_,String>(2)?,"startDate":r.get::<_,Option<String>>(3)?,"endDate":r.get::<_,Option<String>>(4)?}))).map_err(err)?.collect::<std::result::Result<Vec<_>,_>>().map_err(err)?;
            company.relationships = rows.into();
        }
        Ok(companies)
    }
    pub fn save_company(&mut self, mut input: CompanyInput) -> Result<String> {
        if input.name.trim().is_empty() || input.name.len() > 300 || input.notes.len() > 100000 {
            return Err("Enter a company name up to 300 bytes and notes up to 100 KB.".into());
        }
        validate_fields(&input.fields)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().to_rfc3339();
        let previous: Option<(String, String, String)> = tx
            .query_row(
                "SELECT name,fields,notes FROM companies WHERE id=?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(err)?;
        let mut previous_values = previous
            .as_ref()
            .map(|(_, f, _)| serde_json::from_str::<Value>(f))
            .transpose()
            .map_err(err)?
            .unwrap_or(serde_json::json!({}));
        for key in previous_values
            .as_object()
            .ok_or("Invalid stored company facts")?
            .keys()
        {
            if input.fields.get(key).is_none() {
                input.fields[key] = Value::Null;
            }
        }
        if let Some((name, _, notes)) = &previous {
            previous_values["name"] = name.clone().into();
            previous_values["notes"] = notes.clone().into();
        }
        let next = if input.id.is_some() {
            let expected = input
                .expected_revision
                .ok_or("Company revision is required.")?;
            let changed=tx.execute("UPDATE companies SET name=?1,fields=?2,notes=?3,revision=revision+1,provisional=0,updated_at=?4 WHERE id=?5 AND revision=?6 AND merged_into IS NULL",params![input.name.trim(),input.fields.to_string(),input.notes,now,id,expected]).map_err(err)?;
            if changed != 1 {
                return Err("Revision conflict. Reload the company and review your draft.".into());
            }
            expected + 1
        } else {
            tx.execute("INSERT INTO companies(id,name,fields,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?5)",params![id,input.name.trim(),input.fields.to_string(),input.notes,now]).map_err(err)?;
            1
        };
        for (key, value) in input
            .fields
            .as_object()
            .ok_or("Invalid company fields")?
            .iter()
            .chain(
                [
                    ("name".to_string(), Value::String(input.name)),
                    ("notes".to_string(), Value::String(input.notes)),
                ]
                .iter()
                .map(|(k, v)| (k, v)),
            )
        {
            if previous.is_some() && previous_values.get(key) == Some(value) {
                continue;
            }
            tx.execute("INSERT INTO field_overrides VALUES('company',?1,?2,?3,?4) ON CONFLICT(entity_type,entity_id,field) DO UPDATE SET value=excluded.value,revision=excluded.revision",params![id,key,value.to_string(),next]).map_err(err)?;
        }
        tx.commit().map_err(err)?;
        Ok(id)
    }
    pub fn set_employment(
        &mut self,
        company: &str,
        expected: i64,
        kind: &str,
        role: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<String> {
        if !["past", "current"].contains(&kind) || role.len() > 300 {
            return Err("Invalid employment relationship.".into());
        }
        for date in [start, end].into_iter().flatten() {
            let parsed = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(err)?;
            if parsed.format("%Y-%m-%d").to_string() != date {
                return Err("Use YYYY-MM-DD employment dates.".into());
            }
        }
        if start.zip(end).is_some_and(|(a, b)| a > b) || (kind == "current" && end.is_some()) {
            return Err("Invalid employment date range.".into());
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        if tx.execute("UPDATE companies SET revision=revision+1 WHERE id=?1 AND revision=?2 AND merged_into IS NULL",params![company,expected]).map_err(err)?!=1{return Err("Revision conflict. Reload the company.".into());}
        let id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO employment_relationships VALUES(?1,?2,?3,?4,?5,?6,1)",
            params![id, company, kind, role, start, end],
        )
        .map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(id)
    }
    pub fn link_company(&mut self, vacancy: &str, expected: i64, company: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM companies WHERE id=?1 AND merged_into IS NULL)",
                [company],
                |r| r.get(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("Choose an existing company.".into());
        }
        if tx.execute("UPDATE vacancies SET company_id=?1,revision=revision+1 WHERE id=?2 AND revision=?3",params![company,vacancy,expected]).map_err(err)?!=1{return Err("Revision conflict. Reload the vacancy.".into());}
        tx.commit().map_err(err)
    }
    pub fn merge_companies(
        &mut self,
        source: &str,
        target: &str,
        source_revision: i64,
        target_revision: i64,
    ) -> Result<()> {
        if source == target {
            return Err("Choose two different companies.".into());
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let fetch = |id: &str, revision: i64| -> Result<(String, String)> {
            tx.query_row("SELECT fields,notes FROM companies WHERE id=?1 AND revision=?2 AND merged_into IS NULL",params![id,revision],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(err)?.ok_or("Revision conflict. Reload both companies.".into())
        };
        let (source_fields, source_notes) = fetch(source, source_revision)?;
        let (target_fields, target_notes) = fetch(target, target_revision)?;
        let source_value: Value = serde_json::from_str(&source_fields).map_err(err)?;
        let mut target_value: Value = serde_json::from_str(&target_fields).map_err(err)?;
        for (field, value) in source_value.as_object().ok_or("Invalid company facts")? {
            if let Some(target) = target_value.get(field) {
                if target != value {
                    return Err(format!(
                        "Resolve conflicting {field} values before merging."
                    ));
                }
            }
            target_value[field] = value.clone();
        }
        if !source_notes.is_empty() && !target_notes.is_empty() && source_notes != target_notes {
            return Err("Resolve conflicting company notes before merging.".into());
        }
        let protected_notes:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM field_overrides WHERE entity_type='company' AND entity_id=?1 AND field='notes')",[target],|r|r.get(0)).map_err(err)?;
        if protected_notes && target_notes.is_empty() && !source_notes.is_empty() {
            return Err(
                "Target notes are explicitly empty. Resolve that override before merging.".into(),
            );
        }
        if target_notes.is_empty() && !source_notes.is_empty() {
            tx.execute("INSERT OR IGNORE INTO field_overrides SELECT entity_type,?1,field,value,?3 FROM field_overrides WHERE entity_type='company' AND entity_id=?2 AND field='notes'",params![target,source,target_revision+1]).map_err(err)?;
        }
        tx.execute(
            "UPDATE vacancies SET company_id=?1,revision=revision+1 WHERE company_id=?2",
            params![target, source],
        )
        .map_err(err)?;
        for table in ["company_aliases", "company_domains"] {
            tx.execute(
                &format!("UPDATE OR IGNORE {table} SET company_id=?1 WHERE company_id=?2"),
                params![target, source],
            )
            .map_err(err)?;
        }
        tx.execute(
            "UPDATE employment_relationships SET company_id=?1 WHERE company_id=?2",
            params![target, source],
        )
        .map_err(err)?;
        tx.execute("INSERT OR IGNORE INTO entity_sources SELECT entity_type,?1,source_id FROM entity_sources WHERE entity_type='company' AND entity_id=?2",params![target,source]).map_err(err)?;
        for table in ["company_assets", "research_runs", "company_history"] {
            tx.execute(
                &format!("UPDATE {table} SET company_id=?1 WHERE company_id=?2"),
                params![target, source],
            )
            .map_err(err)?;
        }
        tx.execute("INSERT OR IGNORE INTO field_overrides SELECT entity_type,?1,field,value,?3 FROM field_overrides WHERE entity_type='company' AND entity_id=?2 AND field NOT IN ('name','notes')",params![target,source,target_revision+1]).map_err(err)?;
        tx.execute(
            "UPDATE companies SET merged_into=?1,revision=revision+1 WHERE id=?2",
            params![target, source],
        )
        .map_err(err)?;
        tx.execute(
            "UPDATE companies SET fields=?1,notes=?2,revision=revision+1 WHERE id=?3",
            params![
                target_value.to_string(),
                if target_notes.is_empty() {
                    source_notes
                } else {
                    target_notes
                },
                target
            ],
        )
        .map_err(err)?;
        tx.execute(
            "INSERT INTO company_history VALUES(?1,?2,'merge',?3,?4)",
            params![
                uuid::Uuid::new_v4().to_string(),
                target,
                serde_json::json!({"source":source,"target":target}).to_string(),
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(err)?;
        tx.commit().map_err(err)
    }
}
