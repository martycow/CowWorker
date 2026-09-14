use crate::Store;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageFilter {
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub feature: Option<String>,
    pub entity_id: Option<String>,
    pub offset: i64,
    pub limit: i64,
}
impl Store {
    pub fn ai_usage(&self, f: UsageFilter) -> Result<Value, String> {
        let filter=" WHERE (?1 IS NULL OR a.started_at>=?1) AND (?2 IS NULL OR a.started_at<?2) AND (?3 IS NULL OR a.provider=?3) AND (?4 IS NULL OR a.model=?4) AND (?5 IS NULL OR o.operation_type=?5) AND (?6 IS NULL OR o.entity_id=?6)";
        let from=" FROM ai_attempts a JOIN ai_operations o ON o.id=a.operation_id LEFT JOIN usage_measurements u ON u.attempt_id=a.id";
        let p = params![f.from, f.to, f.provider, f.model, f.feature, f.entity_id];
        let totals=self.db.query_row(&format!("SELECT COUNT(DISTINCT o.id),COUNT(*),SUM(u.input_tokens),SUM(u.output_tokens),SUM(u.total_tokens),SUM(CASE WHEN u.actual_micros IS NULL THEN 1 ELSE 0 END){from}{filter}"),p,|r|Ok(json!({"operations":r.get::<_,i64>(0)?,"attempts":r.get::<_,i64>(1)?,"inputTokens":r.get::<_,Option<i64>>(2)?,"outputTokens":r.get::<_,Option<i64>>(3)?,"totalTokens":r.get::<_,Option<i64>>(4)?,"unknownCostAttempts":r.get::<_,Option<i64>>(5)?.unwrap_or(0)}))).map_err(|e|e.to_string())?;
        let mut currency=self.db.prepare(&format!("SELECT u.currency,SUM(u.actual_micros),SUM(u.estimated_micros){from}{filter} GROUP BY u.currency")).map_err(|e|e.to_string())?;
        let costs=currency.query_map(p,|r|Ok(json!({"currency":r.get::<_,Option<String>>(0)?,"actualMicros":r.get::<_,Option<i64>>(1)?,"estimatedMicros":r.get::<_,Option<i64>>(2)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        let mut history=self.db.prepare(&format!("SELECT o.id,o.operation_type,o.entity_id,a.id,a.provider,a.model,a.status,a.started_at,a.latency_ms,u.input_tokens,u.output_tokens,u.total_tokens,u.actual_micros,u.estimated_micros,u.currency,u.quality,a.error{from}{filter} ORDER BY a.started_at DESC,a.rowid DESC LIMIT ?7 OFFSET ?8")).map_err(|e|e.to_string())?;
        let history=history.query_map(params![f.from,f.to,f.provider,f.model,f.feature,f.entity_id,f.limit.clamp(1,100),f.offset.max(0)],|r|Ok(json!({"operationId":r.get::<_,String>(0)?,"feature":r.get::<_,String>(1)?,"entityId":r.get::<_,String>(2)?,"attemptId":r.get::<_,String>(3)?,"provider":r.get::<_,String>(4)?,"model":r.get::<_,String>(5)?,"status":r.get::<_,String>(6)?,"startedAt":r.get::<_,i64>(7)?,"latencyMs":r.get::<_,Option<i64>>(8)?,"inputTokens":r.get::<_,Option<i64>>(9)?,"outputTokens":r.get::<_,Option<i64>>(10)?,"totalTokens":r.get::<_,Option<i64>>(11)?,"actualMicros":r.get::<_,Option<i64>>(12)?,"estimatedMicros":r.get::<_,Option<i64>>(13)?,"currency":r.get::<_,Option<String>>(14)?,"quality":r.get::<_,Option<String>>(15)?,"error":r.get::<_,Option<String>>(16)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        let mut groups = json!({});
        for (name, col) in [
            ("feature", "o.operation_type"),
            ("provider", "a.provider"),
            ("model", "a.model"),
            ("day", "strftime('%Y-%m-%d',a.started_at,'unixepoch')"),
        ] {
            let mut stmt=self.db.prepare(&format!("SELECT {col},COUNT(*),SUM(u.total_tokens){from}{filter} GROUP BY {col} ORDER BY COUNT(*) DESC LIMIT 100")).map_err(|e|e.to_string())?;
            let rows=stmt.query_map(p,|r|Ok(json!({"label":r.get::<_,String>(0)?,"attempts":r.get::<_,i64>(1)?,"tokens":r.get::<_,Option<i64>>(2)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
            groups[name] = rows.into();
        }
        Ok(json!({"totals":totals,"costs":costs,"history":history,"groups":groups}))
    }
    pub fn ai_operations(&self, entity: &str) -> Result<Value, String> {
        let mut stmt=self.db.prepare("SELECT id,operation_type,status,output,error,base_revision,base_version,entity_type FROM ai_operations WHERE entity_id=?1 ORDER BY created_at DESC,rowid DESC LIMIT 50").map_err(|e|e.to_string())?;
        let rows=stmt.query_map([entity],|r|Ok(json!({"id":r.get::<_,String>(0)?,"operationType":r.get::<_,String>(1)?,"status":r.get::<_,String>(2)?,"output":r.get::<_,Option<String>>(3)?.and_then(|s|serde_json::from_str::<Value>(&s).ok()),"error":r.get::<_,Option<String>>(4)?,"baseRevision":r.get::<_,i64>(5)?,"baseVersion":r.get::<_,Option<String>>(6)?,"entityType":r.get::<_,String>(7)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        Ok(rows.into())
    }
    pub fn propose_ai_output(&mut self, id: &str) -> Result<String, String> {
        let existing: Option<String> = self
            .db
            .query_row(
                "SELECT id FROM change_proposals WHERE operation_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(existing) = existing {
            return Ok(existing);
        }
        let (entity,kind,revision,version,output):(String,String,i64,Option<String>,String)=self.db.query_row("SELECT entity_id,entity_type,base_revision,base_version,output FROM ai_operations WHERE id=?1 AND status='completed'",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(|e|e.to_string())?;
        let output: super::AiOutput = serde_json::from_str(&output).map_err(|e| e.to_string())?;
        match kind.as_str() {
            "vacancy" => self.propose_vacancy(
                &entity,
                revision,
                output.fields.ok_or("This analysis has no field changes.")?,
                None,
                Some(id),
            ),
            "document" => self.propose_document_operation(
                &entity,
                revision,
                &version.ok_or("Base version is missing.")?,
                &output
                    .content
                    .ok_or("This analysis has no document changes.")?,
                Some(id),
            ),
            "company" => {
                let fields = serde_json::Value::Object(
                    output.fields.ok_or("This analysis has no field changes.")?,
                );
                crate::companies::validate_fields(&fields)?;
                let proposal = uuid::Uuid::new_v4().to_string();
                self.db.execute("INSERT INTO change_proposals(id,entity_type,entity_id,base_revision,fields,operation_id,created_at) VALUES(?1,'company',?2,?3,?4,?5,?6)",params![proposal,entity,revision,fields.to_string(),id,chrono::Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                Ok(proposal)
            }
            _ => Err(
                "Review company insights with their source evidence before editing company facts."
                    .into(),
            ),
        }
    }
}
