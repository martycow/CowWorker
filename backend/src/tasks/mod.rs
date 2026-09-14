use crate::Store;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn time() -> i64 {
    chrono::Utc::now().timestamp()
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundTask {
    pub id: String,
    pub kind: String,
    pub payload_version: i64,
    pub payload: Value,
    pub status: String,
    pub stage: String,
    pub progress: i64,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub waiting_reason: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub generation: i64,
    pub attempts: i64,
    pub revision: i64,
    pub entity_id: Option<String>,
}
impl Store {
    pub fn run_next_task(&mut self, owner: &str) -> Result<bool> {
        let Some(task) = self.claim_task(owner, time())? else {
            return Ok(false);
        };
        let path = self.path.clone();
        let task_id = task.id.clone();
        let generation = task.generation;
        let owner_id = owner.to_string();
        let (stop, waiting) = std::sync::mpsc::channel::<()>();
        let heartbeat = std::thread::spawn(move || {
            let Ok(mut store) = Store::open(path) else {
                return;
            };
            while matches!(
                waiting.recv_timeout(std::time::Duration::from_secs(5)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ) {
                if store
                    .renew_task_lease(&task_id, &owner_id, generation, time())
                    .is_err()
                {
                    break;
                }
            }
        });
        let result = (|| {
            if task.payload_version != 1 {
                return Err(
                    "Unsupported task payload version. Update CowWorker or create a new task."
                        .into(),
                );
            }
            match task.kind.as_str() {
                "ai-operation" => self.execute_ai(&task, &crate::ai::HttpProvider),
                "company-research" => self.execute_company_research(&task),
                "extract-text" => {
                    let source = task.payload["sourceId"]
                        .as_str()
                        .ok_or("Source ID is required.")?;
                    let bytes = self.source_bytes(source)?;
                    let (source, bytes, media) = if task.payload["mediaType"].as_str()
                        == Some("text/uri-list")
                    {
                        let raw = String::from_utf8(bytes).map_err(|_| "Invalid URL source.")?;
                        let (bytes, media, url) = crate::import::web::fetch(raw.trim())?;
                        let asset =
                            self.acquire_source("Downloaded page", &media, &bytes, Some(&url))?;
                        (asset.id, bytes, media)
                    } else {
                        (source.to_string(), bytes, self.source_media_type(source)?)
                    };
                    let text = if let Some(executable) = &self.extractor {
                        crate::import::process::extract(
                            executable,
                            &self.path.join("sources").join(format!("{source}.bin")),
                            &media,
                            &self.path.join("parser-temp"),
                        )?
                    } else {
                        crate::import::formats::extract(&bytes, &media)?
                    };
                    self.source_snapshot(&source, &text, "bounded-extractor", "1")?;
                    let mut draft = crate::import::classify(
                        &text,
                        task.payload["name"].as_str().unwrap_or("Imported document"),
                    );
                    draft.source_url = task.payload["sourceUrl"].as_str().unwrap_or("").into();
                    Ok(
                        serde_json::json!({"sourceId":source,"draft":draft,"tool":"bounded-extractor","toolVersion":"1"}),
                    )
                }
                "backup" => {
                    let directory = self.path.join("backups").join(&task.id);
                    let manifest = if directory.exists() {
                        crate::backup::verify(&directory)?
                    } else {
                        self.backup(&directory)?
                    };
                    Ok(
                        serde_json::json!({"directory":directory,"schemaVersion":manifest.schema_version,"files":manifest.files.len()}),
                    )
                }
                _ => Err(
                    "This task adapter is not configured. Manual work remains available.".into(),
                ),
            }
        })();
        let finished = self.finish_task(&task, owner, result);
        drop(stop);
        let _ = heartbeat.join();
        finished?;
        Ok(true)
    }
    pub fn renew_task_lease(
        &mut self,
        task: &str,
        owner: &str,
        generation: i64,
        now: i64,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM background_tasks t JOIN runner_leases l ON l.id=1 WHERE t.id=?1 AND t.owner=?2 AND t.generation=?3 AND t.status='running' AND t.cancel_requested=0 AND t.lease_until>?4 AND l.owner=?2 AND l.expires_at>?4)",params![task,owner,generation,now],|r|r.get(0)).map_err(err)?;
        if !valid {
            return Err("Worker lease is no longer current.".into());
        }
        tx.execute(
            "UPDATE background_tasks SET lease_until=?2 WHERE id=?1",
            params![task, now + 30],
        )
        .map_err(err)?;
        tx.execute(
            "UPDATE runner_leases SET expires_at=?1 WHERE id=1",
            [now + 30],
        )
        .map_err(err)?;
        tx.commit().map_err(err)
    }
    pub fn enqueue_task(
        &mut self,
        kind: &str,
        payload: Value,
        key: &str,
        entity: Option<&str>,
    ) -> Result<String> {
        if !["extract-text", "backup", "company-research", "ai-operation"].contains(&kind) {
            return Err("Unsupported task type.".into());
        }
        if key.is_empty() || key.len() > 300 || payload.to_string().len() > 16000 {
            return Err("Invalid task payload or idempotency key.".into());
        }
        let old: Option<(String, String, String)> = self
            .db
            .query_row(
                "SELECT id,kind,payload FROM background_tasks WHERE idempotency_key=?1",
                [key],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(err)?;
        if let Some((id, old_kind, old_payload)) = old {
            let serialized = payload.to_string();
            if old_kind != kind || old_payload != serialized {
                return Err("Idempotency key was used for different input.".into());
            }
            return Ok(id);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let now = time();
        self.db.execute("INSERT INTO background_tasks(id,kind,payload,idempotency_key,created_at,updated_at,entity_id) VALUES(?1,?2,?3,?4,?5,?5,?6)",params![id,kind,payload.to_string(),key,now,entity]).map_err(err)?;
        Ok(id)
    }
    pub fn task_dependency(&mut self, task: &str, dependency: &str, required: bool) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let started: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM background_tasks WHERE id=?1 AND attempts>0)",
                [task],
                |r| r.get(0),
            )
            .map_err(err)?;
        if started {
            return Err("Dependencies must be set before execution.".into());
        }
        let cycle:bool=tx.query_row("WITH RECURSIVE ancestors(id) AS (SELECT ?1 UNION SELECT depends_on FROM task_dependencies JOIN ancestors ON task_id=ancestors.id) SELECT EXISTS(SELECT 1 FROM ancestors WHERE id=?2)",params![dependency,task],|r|r.get(0)).map_err(err)?;
        if cycle {
            return Err("Task dependency cycle is not allowed.".into());
        }
        tx.execute(
            "INSERT INTO task_dependencies VALUES(?1,?2,?3)",
            params![task, dependency, required],
        )
        .map_err(err)?;
        tx.commit().map_err(err)
    }
    pub fn tasks(&self, offset: i64, limit: i64) -> Result<Vec<BackgroundTask>> {
        let mut stmt=self.db.prepare("SELECT id,kind,payload_version,payload,status,stage,progress,result,error,waiting_reason,created_at,updated_at,generation,attempts,revision,entity_id FROM background_tasks ORDER BY created_at DESC,rowid DESC LIMIT ?1 OFFSET ?2").map_err(err)?;
        let rows = stmt
            .query_map(params![limit.clamp(1, 100), offset.max(0)], |r| {
                Ok(BackgroundTask {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    payload_version: r.get(2)?,
                    payload: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default(),
                    status: r.get(4)?,
                    stage: r.get(5)?,
                    progress: r.get(6)?,
                    result: r
                        .get::<_, Option<String>>(7)?
                        .map(|s| serde_json::from_str(&s).unwrap_or_default()),
                    error: r.get(8)?,
                    waiting_reason: r.get(9)?,
                    created_at: r.get(10)?,
                    updated_at: r.get(11)?,
                    generation: r.get(12)?,
                    attempts: r.get(13)?,
                    revision: r.get(14)?,
                    entity_id: r.get(15)?,
                })
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        Ok(rows)
    }
    pub fn cancel_task(&mut self, id: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        tx.execute("UPDATE background_tasks SET cancel_requested=1,status='cancelled',stage='Cancelled',revision=revision+1,updated_at=?2 WHERE id=?1 AND status IN ('queued','running','waiting')",params![id,time()]).map_err(err)?;
        tx.execute("UPDATE task_attempts SET status='cancelled',finished_at=?2 WHERE task_id=?1 AND status='running'",params![id,time()]).map_err(err)?;
        tx.commit().map_err(err)
    }
    pub fn retry_task(&mut self, id: &str) -> Result<()> {
        let dispatched: bool = self.db.query_row("SELECT EXISTS(SELECT 1 FROM ai_attempts a JOIN ai_operations o ON o.id=a.operation_id WHERE o.task_id=?1)",[id],|r|r.get(0)).map_err(err)?;
        if dispatched {
            return Err("This model operation was already dispatched. Review its usage and authorize a new operation in the AI panel.".into());
        }
        let changed=self.db.execute("UPDATE background_tasks SET status='queued',stage='Retry queued',error=NULL,waiting_reason=NULL,cancel_requested=0,next_attempt=0,revision=revision+1,updated_at=?2 WHERE id=?1 AND status IN ('failed','cancelled')",params![id,time()]).map_err(err)?;
        if changed != 1 {
            return Err("Only failed or cancelled tasks can be retried. Unknown remote outcomes need reconciliation.".into());
        }
        Ok(())
    }
    pub fn claim_task(&mut self, owner: &str, now: i64) -> Result<Option<BackgroundTask>> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        tx.execute("INSERT INTO runner_leases VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET owner=excluded.owner,expires_at=excluded.expires_at WHERE runner_leases.owner=?1 OR runner_leases.expires_at<=?3",params![owner,now+30,now]).map_err(err)?;
        let active: String = tx
            .query_row("SELECT owner FROM runner_leases WHERE id=1", [], |r| {
                r.get(0)
            })
            .map_err(err)?;
        if active != owner {
            return Ok(None);
        }
        tx.execute("UPDATE task_attempts SET status='interrupted',finished_at=?1 WHERE status='running' AND task_id IN (SELECT id FROM background_tasks WHERE status='running' AND lease_until<=?1)",[now]).map_err(err)?;
        tx.execute("UPDATE ai_attempts SET status='unknown',finished_at=?1,error='Process interrupted after dispatch; usage is unknown.' WHERE status='running' AND operation_id IN (SELECT o.id FROM ai_operations o JOIN background_tasks t ON t.id=o.task_id WHERE t.status='running' AND t.lease_until<=?1)",[now]).map_err(err)?;
        tx.execute("INSERT OR IGNORE INTO usage_measurements(attempt_id,quality) SELECT id,'unknown' FROM ai_attempts WHERE status='unknown'",[]).map_err(err)?;
        tx.execute("UPDATE ai_operations SET status='waiting',error='Unknown remote outcome. Reconciliation is required.',updated_at=?1 WHERE task_id IN (SELECT id FROM background_tasks WHERE kind='ai-operation' AND status='running' AND lease_until<=?1)",[now]).map_err(err)?;
        tx.execute("UPDATE background_tasks SET status=CASE WHEN kind='ai-operation' THEN 'waiting' ELSE 'queued' END,waiting_reason=CASE WHEN kind='ai-operation' THEN 'unknown-remote-outcome' ELSE NULL END,stage='Recovered after interruption',revision=revision+1,updated_at=?1 WHERE status='running' AND lease_until<=?1",[now]).map_err(err)?;
        tx.execute("UPDATE background_tasks SET status='failed',error='A required dependency failed or was cancelled.',stage='Dependency failed',revision=revision+1,updated_at=?1 WHERE status='queued' AND EXISTS(SELECT 1 FROM task_dependencies d JOIN background_tasks p ON p.id=d.depends_on WHERE d.task_id=background_tasks.id AND d.required=1 AND p.status IN ('failed','cancelled'))",[now]).map_err(err)?;
        let next:Option<String>=tx.query_row("SELECT id FROM background_tasks t WHERE status='queued' AND next_attempt<=?1 AND cancel_requested=0 AND NOT EXISTS(SELECT 1 FROM task_dependencies d JOIN background_tasks p ON p.id=d.depends_on WHERE d.task_id=t.id AND p.status NOT IN ('completed','failed','cancelled')) ORDER BY created_at,rowid LIMIT 1",[now],|r|r.get(0)).optional().map_err(err)?;
        let Some(id) = next else {
            tx.commit().map_err(err)?;
            return Ok(None);
        };
        tx.execute("UPDATE background_tasks SET status='running',stage='Processing',owner=?2,lease_until=?3,generation=generation+1,attempts=attempts+1,revision=revision+1,updated_at=?4 WHERE id=?1",params![id,owner,now+30,now]).map_err(err)?;
        tx.execute("INSERT INTO task_attempts SELECT ?1,id,generation,?2,NULL,'running',NULL FROM background_tasks WHERE id=?3",params![uuid::Uuid::new_v4().to_string(),now,id]).map_err(err)?;
        tx.commit().map_err(err)?;
        self.task_by_id(&id).map(Some)
    }
    pub fn task_by_id(&self, id: &str) -> Result<BackgroundTask> {
        let rowid: i64 = self
            .db
            .query_row(
                "SELECT rowid FROM background_tasks WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(err)?;
        let offset:i64=self.db.query_row("SELECT COUNT(*) FROM background_tasks WHERE created_at>(SELECT created_at FROM background_tasks WHERE id=?1) OR (created_at=(SELECT created_at FROM background_tasks WHERE id=?1) AND rowid>?2)",params![id,rowid],|r|r.get(0)).map_err(err)?;
        self.tasks(offset, 1)?
            .into_iter()
            .next()
            .ok_or_else(|| "Task no longer exists.".into())
    }
    pub fn finish_task(
        &mut self,
        task: &BackgroundTask,
        owner: &str,
        result: std::result::Result<Value, String>,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(err)?;
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM background_tasks t JOIN runner_leases l ON l.id=1 WHERE t.id=?1 AND t.owner=?2 AND t.generation=?3 AND t.status='running' AND t.cancel_requested=0 AND t.lease_until>?4 AND l.owner=?2 AND l.expires_at>?4)",params![task.id,owner,task.generation,time()],|r|r.get(0)).map_err(err)?;
        if !valid {
            return Err(
                "Task was cancelled or its worker lease expired. Result was not committed.".into(),
            );
        }
        let (status, result, error) = match result {
            Ok(value) => ("completed", Some(value.to_string()), None),
            Err(error) => ("failed", None, Some(error)),
        };
        if task.kind == "company-research" {
            tx.execute(
                "UPDATE research_runs SET status=?2,error=?3,completed_at=?4 WHERE task_id=?1",
                params![task.id, status, error, time()],
            )
            .map_err(err)?;
            if let Some(result) = &result {
                let value: Value = serde_json::from_str(result).map_err(err)?;
                let company = value["companyId"]
                    .as_str()
                    .ok_or("Company result is missing its target.")?;
                let source = value["sourceId"]
                    .as_str()
                    .ok_or("Company result is missing its source.")?;
                tx.execute(
                    "UPDATE research_runs SET source_id=?1 WHERE task_id=?2",
                    params![source, task.id],
                )
                .map_err(err)?;
                tx.execute(
                    "INSERT OR IGNORE INTO entity_sources VALUES('company',?1,?2)",
                    params![company, source],
                )
                .map_err(err)?;
                tx.execute("INSERT INTO change_proposals(id,entity_type,entity_id,base_revision,fields,source_id,created_at) VALUES(?1,'company',?2,?3,?4,?5,?6)",params![uuid::Uuid::new_v4().to_string(),company,value["baseRevision"].as_i64().ok_or("Missing research revision")?,value["fields"].to_string(),source,chrono::Utc::now().to_rfc3339()]).map_err(err)?;
            }
        }
        tx.execute("INSERT INTO task_checkpoints VALUES(?1,'result',?2) ON CONFLICT(task_id,stage) DO UPDATE SET value=excluded.value",params![task.id,result.as_deref().unwrap_or("null")]).map_err(err)?;
        tx.execute("UPDATE background_tasks SET status=?2,stage=?2,progress=CASE WHEN ?2='completed' THEN 100 ELSE progress END,result=?3,error=?4,revision=revision+1,updated_at=?5 WHERE id=?1",params![task.id,status,result,error,time()]).map_err(err)?;
        tx.execute("UPDATE task_attempts SET status=?3,error=?4,finished_at=?5 WHERE task_id=?1 AND generation=?2",params![task.id,task.generation,status,error,time()]).map_err(err)?;
        tx.commit().map_err(err)
    }
}
