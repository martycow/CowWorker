use cowworker_core::*;
use serde_json::{json, Map, Value};
fn fields(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}
fn vacancy(store: &mut Store) -> String {
    store.save_vacancy(serde_json::from_value(json!({"title":"Role","company":"Company","location":"","workMode":"Remote","sourceUrl":"","description":"original","notes":"manual","status":"saved"})).unwrap()).unwrap()
}
#[test]
fn empty_override_stale_results_and_clear_are_guarded() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let id = vacancy(&mut s);
    s.patch_vacancy(&id, 1, fields(json!({"notes":""})))
        .unwrap();
    let p = s
        .propose_vacancy(&id, 2, fields(json!({"notes":"candidate"})), None, None)
        .unwrap();
    assert!(s
        .review_proposal(&p, true, false)
        .unwrap_err()
        .contains("override"));
    assert_eq!(s.workspace().unwrap().vacancies[0].notes, "");
    s.review_proposal(&p, true, true).unwrap();
    s.patch_vacancy(&id, 3, fields(json!({"notes":"manual again"})))
        .unwrap();
    s.clear_override(&id, 4, "notes").unwrap();
    assert_eq!(s.workspace().unwrap().vacancies[0].notes, "candidate");
    assert!(s
        .patch_vacancy(&id, 4, fields(json!({"status":"archived"})))
        .unwrap_err()
        .contains("Revision conflict"));
    let stale = s
        .propose_vacancy(&id, 2, fields(json!({"notes":"late"})), None, None)
        .unwrap();
    assert!(s
        .review_proposal(&stale, true, true)
        .unwrap_err()
        .contains("Revision conflict"));
    assert_eq!(
        s.proposals(&id)
            .unwrap()
            .iter()
            .find(|p| p.id == stale)
            .unwrap()
            .status,
        "review"
    );
}
#[test]
fn two_connections_cannot_overwrite_newer_fields() {
    let dir = tempfile::tempdir().unwrap();
    let mut a = Store::open(dir.path()).unwrap();
    let id = vacancy(&mut a);
    let mut b = Store::open(dir.path()).unwrap();
    a.patch_vacancy(&id, 1, fields(json!({"notes":"fresh"})))
        .unwrap();
    assert!(b
        .patch_vacancy(&id, 1, fields(json!({"notes":"stale"})))
        .is_err());
    b.patch_vacancy(&id, 2, fields(json!({"status":"shortlisted"})))
        .unwrap();
    let row = &a.workspace().unwrap().vacancies[0];
    assert_eq!(row.notes, "fresh");
    assert_eq!(row.status, "shortlisted");
}
#[test]
fn source_bytes_and_crash_leftovers_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let id = vacancy(&mut s);
    let source = s
        .acquire_source("original.txt", "text/plain", "Текст\r\n".as_bytes(), None)
        .unwrap();
    s.link_source("vacancy", &id, &source.id).unwrap();
    s.source_snapshot(&source.id, "Текст", "utf8", "1").unwrap();
    assert_eq!(s.source_bytes(&source.id).unwrap(), "Текст\r\n".as_bytes());
    std::fs::write(dir.path().join("sources/orphan.tmp"), "retain me").unwrap();
    assert_eq!(s.reconcile_assets().unwrap(), vec!["sources/orphan.tmp"]);
    assert_eq!(s.entity_sources(&id).unwrap().len(), 1);
    let out = tempfile::tempdir().unwrap();
    s.backup(&out.path().join("backup")).unwrap();
    backup::restore(&out.path().join("backup"), &out.path().join("restored")).unwrap();
    let restored = Store::open(out.path().join("restored")).unwrap();
    assert_eq!(
        restored.source_bytes(&source.id).unwrap(),
        "Текст\r\n".as_bytes()
    );
}
#[test]
fn document_proposals_create_versions_and_category_correction_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let doc = s
        .save_document(DocumentInput {
            id: None,
            expected_revision: None,
            title: "Resume".into(),
            kind: "resume".into(),
            content: "original".into(),
        })
        .unwrap();
    let base = s.workspace().unwrap().documents[0].versions[0].id.clone();
    let p = s.propose_document(&doc, 1, &base, "reviewed").unwrap();
    s.accept_document_proposal(&p).unwrap();
    assert!(s.accept_document_proposal(&p).is_err());
    s.correct_document(&doc, 2, "Reference", "reference")
        .unwrap();
    let data = s.workspace().unwrap();
    assert_eq!(data.documents[0].versions.len(), 2);
    assert_eq!(data.documents[0].kind, "reference");
    assert_eq!(s.document_content(&base).unwrap(), "original");
    let stale = s.propose_document(&doc, 1, &base, "stale").unwrap();
    assert!(s.accept_document_proposal(&stale).is_err());
}
