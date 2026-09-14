use crate::{BackgroundTask, Store};
use rusqlite::{params, TransactionBehavior};
use serde_json::{json, Value};
impl Store {
    pub fn start_company_research(
        &mut self,
        company: &str,
        expected: i64,
        url: &str,
    ) -> Result<String, String> {
        let parsed = url::Url::parse(url).map_err(|e| e.to_string())?;
        if !["http", "https"].contains(&parsed.scheme())
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            return Err("Use a public website without credentials.".into());
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let current: i64 = tx
            .query_row(
                "SELECT revision FROM companies WHERE id=?1 AND merged_into IS NULL",
                [company],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if current != expected {
            return Err("Revision conflict. Reload the company before researching.".into());
        }
        let task = uuid::Uuid::new_v4().to_string();
        let run = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        tx.execute("INSERT INTO background_tasks(id,kind,payload,idempotency_key,created_at,updated_at,entity_id) VALUES(?1,'company-research',?2,?3,?4,?4,?5)",params![task,json!({"companyId":company,"baseRevision":expected,"url":url,"runId":run}).to_string(),format!("research:{run}"),now,company]).map_err(|e|e.to_string())?;
        tx.execute("INSERT INTO research_runs(id,company_id,task_id,base_revision,source_url,created_at) VALUES(?1,?2,?3,?4,?5,?6)",params![run,company,task,expected,url,now]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(task)
    }
    pub fn execute_company_research(&mut self, task: &BackgroundTask) -> Result<Value, String> {
        let url = task.payload["url"]
            .as_str()
            .ok_or("Research URL is missing.")?;
        let (bytes, media, final_url) = crate::import::web::fetch(url)?;
        let source = self.acquire_source("Company website", &media, &bytes, Some(&final_url))?;
        let text = crate::import::formats::extract(&bytes, &media)?;
        self.source_snapshot(&source.id, &text, "website-text", "1")?;
        let html = scraper::Html::parse_document(
            std::str::from_utf8(&bytes).map_err(|_| "Website is not UTF-8.")?,
        );
        let description =
            scraper::Selector::parse("meta[name='description'],meta[property='og:description']")
                .map_err(|e| e.to_string())?;
        let summary = html
            .select(&description)
            .filter_map(|e| e.value().attr("content"))
            .find(|s| !s.trim().is_empty())
            .map(|s| s.chars().take(10000).collect::<String>());
        let mut fields = json!({"website":final_url});
        if let Some(summary) = summary {
            fields["description"] = summary.into();
        }
        // Website metadata is a candidate, not a verified fact or an AI-generated result.
        Ok(
            json!({"companyId":task.payload["companyId"],"baseRevision":task.payload["baseRevision"],"runId":task.payload["runId"],"sourceId":source.id,"fields":fields,"observedAt":chrono::Utc::now().timestamp()}),
        )
    }
    pub fn company_research_history(&self, company: &str) -> Result<Value, String> {
        let mut stmt=self.db.prepare("SELECT id,status,source_url,source_id,error,created_at,completed_at FROM research_runs WHERE company_id=?1 ORDER BY created_at DESC,rowid DESC LIMIT 25").map_err(|e|e.to_string())?;
        let rows=stmt.query_map([company],|r|Ok(json!({"id":r.get::<_,String>(0)?,"status":r.get::<_,String>(1)?,"sourceUrl":r.get::<_,String>(2)?,"sourceId":r.get::<_,Option<String>>(3)?,"error":r.get::<_,Option<String>>(4)?,"createdAt":r.get::<_,i64>(5)?,"completedAt":r.get::<_,Option<i64>>(6)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        Ok(rows.into())
    }
}
