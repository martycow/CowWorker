CREATE TABLE provider_settings(id INTEGER PRIMARY KEY CHECK(id=1),value TEXT NOT NULL);
CREATE UNIQUE INDEX ai_proposal_operation ON change_proposals(operation_id) WHERE operation_id IS NOT NULL;
CREATE TABLE ai_operations(id TEXT PRIMARY KEY,operation_type TEXT NOT NULL,entity_type TEXT NOT NULL,entity_id TEXT NOT NULL,base_revision INTEGER NOT NULL,base_version TEXT,provider TEXT NOT NULL,model TEXT NOT NULL,request TEXT NOT NULL,authorization_scope TEXT NOT NULL,status TEXT NOT NULL,task_id TEXT REFERENCES background_tasks(id),output TEXT,error TEXT,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL,idempotency_key TEXT NOT NULL UNIQUE);
CREATE TABLE ai_attempts(id TEXT PRIMARY KEY,operation_id TEXT NOT NULL REFERENCES ai_operations(id),provider TEXT NOT NULL,model TEXT NOT NULL,status TEXT NOT NULL,started_at INTEGER NOT NULL,finished_at INTEGER,latency_ms INTEGER,request_id TEXT,error TEXT,task_generation INTEGER NOT NULL,UNIQUE(operation_id,task_generation));
CREATE TABLE usage_measurements(attempt_id TEXT PRIMARY KEY REFERENCES ai_attempts(id),input_tokens INTEGER,output_tokens INTEGER,total_tokens INTEGER,estimated_micros INTEGER,actual_micros INTEGER,currency TEXT,quality TEXT NOT NULL,price_snapshot TEXT);
CREATE TABLE price_snapshots(id TEXT PRIMARY KEY,provider TEXT NOT NULL,model TEXT NOT NULL,currency TEXT NOT NULL,input_micros_per_million INTEGER,output_micros_per_million INTEGER,recorded_at INTEGER NOT NULL);
INSERT INTO schema_migrations VALUES(6,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
PRAGMA user_version=6;
