use cowworker_core::*;
use serde_json::json;
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
#[test]
fn heartbeat_keeps_current_worker_and_rejects_cancelled_or_expired_owners() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let id = store
        .enqueue_task("backup", json!({}), "long", None)
        .unwrap();
    let started = now();
    let task = store.claim_task("first", started).unwrap().unwrap();
    store
        .renew_task_lease(&id, "first", task.generation, started + 20)
        .unwrap();
    let mut second = Store::open(dir.path()).unwrap();
    assert!(second.claim_task("second", started + 31).unwrap().is_none());
    assert!(store
        .renew_task_lease(&id, "second", task.generation, started + 32)
        .is_err());
    store.cancel_task(&id).unwrap();
    assert!(store
        .renew_task_lease(&id, "first", task.generation, started + 32)
        .is_err());
}
#[test]
fn five_inputs_finish_independently_and_idempotency_is_checked() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    for i in 0..5 {
        let source = s
            .acquire_source(
                "input.txt",
                "text/plain",
                if i == 2 { b"bad\0file" } else { b"good text" },
                None,
            )
            .unwrap();
        let payload = json!({"sourceId":source.id});
        let key = format!("input-{i}");
        let id = s
            .enqueue_task("extract-text", payload.clone(), &key, None)
            .unwrap();
        assert_eq!(
            id,
            s.enqueue_task("extract-text", payload, &key, None).unwrap()
        );
    }
    for _ in 0..5 {
        assert!(s.run_next_task("worker").unwrap());
    }
    let tasks = s.tasks(0, 100).unwrap();
    assert_eq!(tasks.iter().filter(|t| t.status == "completed").count(), 4);
    assert_eq!(tasks.iter().filter(|t| t.status == "failed").count(), 1);
    assert!(s
        .enqueue_task("backup", json!({}), "input-0", None)
        .is_err());
}
#[test]
fn cancellation_and_expired_generation_fence_old_results() {
    let dir = tempfile::tempdir().unwrap();
    let mut a = Store::open(dir.path()).unwrap();
    let mut b = Store::open(dir.path()).unwrap();
    let id = a.enqueue_task("backup", json!({}), "backup", None).unwrap();
    let old = a.claim_task("old", now()).unwrap().unwrap();
    assert!(b.claim_task("other", now()).unwrap().is_none());
    b.cancel_task(&id).unwrap();
    assert!(a
        .finish_task(&old, "old", Ok(json!({"wrong":true})))
        .is_err());
    b.retry_task(&id).unwrap();
    let fresh = b.claim_task("new", now() + 31).unwrap().unwrap();
    assert!(fresh.generation > old.generation);
    assert!(a.finish_task(&old, "old", Ok(json!({}))).is_err());
    b.finish_task(&fresh, "new", Ok(json!({"saved":true})))
        .unwrap();
    b.cancel_task(&id).unwrap();
    assert_eq!(b.task_by_id(&id).unwrap().status, "completed");
}
#[test]
fn dependency_cycles_and_required_failures_are_visible() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let a = s.enqueue_task("backup", json!({}), "a", None).unwrap();
    let b = s.enqueue_task("backup", json!({}), "b", None).unwrap();
    s.task_dependency(&b, &a, true).unwrap();
    assert!(s.task_dependency(&a, &b, true).is_err());
    s.cancel_task(&a).unwrap();
    assert!(s.claim_task("worker", now()).unwrap().is_none());
    assert_eq!(s.task_by_id(&b).unwrap().status, "failed");
}
#[test]
fn interrupted_remote_outcome_waits_instead_of_redispatching() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let id = s
        .enqueue_task("ai-operation", json!({}), "remote", None)
        .unwrap();
    s.claim_task("old", now()).unwrap().unwrap();
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert!(s.claim_task("new", now() + 31).unwrap().is_none());
    assert_eq!(
        s.task_by_id(&id).unwrap().waiting_reason.as_deref(),
        Some("unknown-remote-outcome")
    );
    assert!(s.retry_task(&id).is_err());
}
