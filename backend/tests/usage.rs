use cowworker_core::{ai::UsageFilter, Store};
use rusqlite::Connection;

#[test]
fn synthetic_ledger_preserves_unknowns_currencies_and_pagination() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
    db.execute_batch("BEGIN;
      WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<10000)
      INSERT INTO ai_operations(id,operation_type,entity_type,entity_id,base_revision,provider,model,request,authorization_scope,status,created_at,updated_at,idempotency_key)
      SELECT 'op-'||i,CASE WHEN i%2=0 THEN 'job-analysis' ELSE 'company-research' END,'vacancy','v',1,'fixture','m','{}','scope','completed',i,i,'key-'||i FROM n;
      INSERT INTO ai_attempts(id,operation_id,provider,model,status,started_at,task_generation) SELECT 'a-'||id,id,provider,model,status,created_at,1 FROM ai_operations;
      INSERT INTO usage_measurements(attempt_id,input_tokens,output_tokens,total_tokens,actual_micros,currency,quality)
      SELECT id,10,5,15,CASE WHEN started_at%3=0 THEN NULL ELSE 100 END,CASE WHEN started_at%2=0 THEN 'USD' ELSE 'EUR' END,CASE WHEN started_at%3=0 THEN 'unknown' ELSE 'reported' END FROM ai_attempts;
      COMMIT;").unwrap();
    let start = std::time::Instant::now();
    let all = store
        .ai_usage(UsageFilter {
            limit: 25,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(all["totals"]["operations"], 10000);
    assert_eq!(all["totals"]["attempts"], 10000);
    assert_eq!(all["totals"]["totalTokens"], 150000);
    assert_eq!(all["totals"]["unknownCostAttempts"], 3333);
    assert_eq!(all["costs"].as_array().unwrap().len(), 2);
    assert_eq!(all["history"].as_array().unwrap().len(), 25);
    let page = store
        .ai_usage(UsageFilter {
            limit: 25,
            offset: 25,
            ..Default::default()
        })
        .unwrap();
    assert_ne!(
        all["history"][0]["attemptId"],
        page["history"][0]["attemptId"]
    );
    let range = store
        .ai_usage(UsageFilter {
            from: Some(100),
            to: Some(200),
            limit: 100,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(range["totals"]["operations"], 100);
    assert_eq!(range["totals"]["totalTokens"], 1500);
    assert_eq!(range["history"].as_array().unwrap().len(), 100);
    println!(
        "10,000-attempt ledger, all/page/date queries: {:?}",
        start.elapsed()
    );
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
}
