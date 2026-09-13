use cowworker_core::*;

fn vacancy() -> VacancyInput {
    VacancyInput {
        id: None,
        title: "Frontend Engineer".into(),
        company: "Example".into(),
        location: "US".into(),
        work_mode: "Remote".into(),
        source_url: "https://example.com/jobs/1".into(),
        description: "Original posting".into(),
        notes: "Ask about accessibility".into(),
        status: "saved".into(),
    }
}
fn document(id: Option<String>, content: &str) -> DocumentInput {
    DocumentInput {
        id,
        title: "Resume".into(),
        kind: "resume".into(),
        content: content.into(),
    }
}
fn app_input(app: &Application) -> ApplicationInput {
    ApplicationInput {
        id: app.id.clone(),
        stage: app.stage.clone(),
        next_action: app.next_action.clone(),
        due_date: app.due_date.clone(),
        notes: app.notes.clone(),
        document_version_ids: app.document_version_ids.clone(),
    }
}

#[test]
fn vacancy_to_application_survives_restart_with_exact_submitted_copy() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let vacancy_id = store.save_vacancy(vacancy()).unwrap();
    let doc_id = store
        .save_document(document(None, "Resume v1\r\nТочный текст"))
        .unwrap();
    let version_id = store.workspace().unwrap().documents[0].versions[0]
        .id
        .clone();
    let app_id = store.prepare_application(&vacancy_id).unwrap();
    assert_eq!(app_id, store.prepare_application(&vacancy_id).unwrap());
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.stage = "applied".into();
    input.document_version_ids = vec![version_id.clone()];
    input.next_action = "Follow up".into();
    input.due_date = "2026-09-20".into();
    store.save_application(input).unwrap();
    store
        .save_document(document(Some(doc_id), "Resume v2"))
        .unwrap();
    drop(store);
    let reopened = Store::open(dir.path()).unwrap();
    let data = reopened.workspace().unwrap();
    assert_eq!(data.vacancies[0].description, "Original posting");
    assert_eq!(data.applications.len(), 1);
    assert_eq!(data.applications[0].stage, "applied");
    assert!(data.applications[0].submitted_at.is_some());
    assert_eq!(data.applications[0].events.len(), 2);
    assert_eq!(data.applications[0].due_date, "2026-09-20");
    assert_eq!(data.documents[0].versions.len(), 2);
    assert_eq!(
        data.applications[0].document_version_ids,
        vec![version_id.clone()]
    );
    let sent = data.documents[0]
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .unwrap();
    assert_eq!(sent.content, "Resume v1\r\nТочный текст");
    assert_eq!(
        std::fs::read_to_string(
            dir.path()
                .join("documents")
                .join(format!("{version_id}.txt"))
        )
        .unwrap(),
        sent.content
    );
}

#[test]
fn submission_guards_are_atomic() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let v = store.save_vacancy(vacancy()).unwrap();
    store.prepare_application(&v).unwrap();
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.stage = "applied".into();
    assert!(store
        .save_application(input)
        .unwrap_err()
        .contains("Attach"));
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.stage = "applied".into();
    input.document_version_ids = vec!["missing".into()];
    assert!(store
        .save_application(input)
        .unwrap_err()
        .contains("no longer exists"));
    let app = &store.workspace().unwrap().applications[0];
    assert_eq!(app.stage, "preparing");
    assert_eq!(app.events.len(), 1);
    assert!(app.document_version_ids.is_empty());
}

#[test]
fn submitted_versions_and_preparation_cannot_be_rewritten() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let v = store.save_vacancy(vacancy()).unwrap();
    store.prepare_application(&v).unwrap();
    store.save_document(document(None, "Version 1")).unwrap();
    let data = store.workspace().unwrap();
    let mut input = app_input(&data.applications[0]);
    input.stage = "interview".into();
    input.document_version_ids = vec![data.documents[0].versions[0].id.clone()];
    store.save_application(input).unwrap();
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.document_version_ids.clear();
    assert!(store
        .save_application(input)
        .unwrap_err()
        .contains("locked"));
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.stage = "preparing".into();
    assert!(store
        .save_application(input)
        .unwrap_err()
        .contains("cannot return"));
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.stage = "offer".into();
    store.save_application(input).unwrap();
    assert_eq!(store.workspace().unwrap().applications[0].events.len(), 3);
}

#[test]
fn rejects_duplicate_urls_unsafe_sources_and_invalid_fields() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store.save_vacancy(vacancy()).unwrap();
    let mut input = vacancy();
    input.source_url += "#details";
    assert!(store
        .save_vacancy(input)
        .unwrap_err()
        .contains("already saved"));
    for source in [
        "javascript:alert(1)",
        "file:///C:/secret",
        "https://user:password@example.com",
    ] {
        let mut input = vacancy();
        input.source_url = source.into();
        assert!(store.save_vacancy(input).is_err());
    }
    let mut input = vacancy();
    input.title = "  ".into();
    assert!(store.save_vacancy(input).is_err());
    let mut input = vacancy();
    input.status = "invented".into();
    assert!(store.save_vacancy(input).is_err());
    let mut input = vacancy();
    input.id = Some("missing".into());
    input.source_url.clear();
    assert!(store
        .save_vacancy(input)
        .unwrap_err()
        .contains("no longer exists"));
    assert_eq!(store.workspace().unwrap().vacancies.len(), 1);
}

#[test]
fn archive_restore_profile_and_date_validation() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut input = vacancy();
    input.status = "archived".into();
    let v = store.save_vacancy(input).unwrap();
    assert!(store.prepare_application(&v).is_err());
    let mut input = vacancy();
    input.id = Some(v.clone());
    store.save_vacancy(input).unwrap();
    store.prepare_application(&v).unwrap();
    for date in ["2026-02-30", "2026-2-1", "next week"] {
        let mut input = app_input(&store.workspace().unwrap().applications[0]);
        input.due_date = date.into();
        assert!(store.save_application(input).is_err());
    }
    let mut input = app_input(&store.workspace().unwrap().applications[0]);
    input.due_date = "2026-09-20".into();
    input.next_action.clear();
    assert!(store.save_application(input).is_err());
    store
        .save_profile(Profile {
            name: "Marty".into(),
            headline: "Engineer".into(),
            email: "marty@example.com".into(),
            summary: "Facts".into(),
        })
        .unwrap();
    drop(store);
    assert_eq!(
        Store::open(dir.path())
            .unwrap()
            .workspace()
            .unwrap()
            .profile
            .name,
        "Marty"
    );
}

#[test]
fn future_schema_and_missing_files_fail_explicitly() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store.save_document(document(None, "Keep me")).unwrap();
    let id = store.workspace().unwrap().documents[0].versions[0]
        .id
        .clone();
    std::fs::remove_file(dir.path().join("documents").join(format!("{id}.txt"))).unwrap();
    assert!(store.workspace().unwrap_err().contains("Restore its file"));
    drop(store);
    let db = rusqlite::Connection::open(dir.path().join("cowworker.db")).unwrap();
    db.execute_batch("PRAGMA user_version=99;").unwrap();
    assert!(Store::open(dir.path()).is_err());
}
