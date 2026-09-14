use cowworker_core::*;
#[test]
fn schema_eight_import_review_is_backfilled_and_remains_editable() {
    use rusqlite::{params, Connection};
    use sha2::{Digest, Sha256};
    let dir = tempfile::tempdir().unwrap();
    let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
    for sql in [
        include_str!("../src/schema.sql"),
        include_str!("../src/migrations/002.sql"),
        include_str!("../src/migrations/003.sql"),
        include_str!("../src/migrations/004.sql"),
        include_str!("../src/migrations/005.sql"),
        include_str!("../src/migrations/006.sql"),
        include_str!("../src/migrations/007.sql"),
        include_str!("../src/migrations/008.sql"),
    ] {
        db.execute_batch(sql).unwrap();
    }
    let source = uuid::Uuid::new_v4().to_string();
    let bytes = b"Experience\nOriginal career facts\nEducation\nExample University";
    let name = format!("{source}.bin");
    std::fs::create_dir(dir.path().join("sources")).unwrap();
    std::fs::write(dir.path().join("sources").join(&name), bytes).unwrap();
    db.execute(
        "INSERT INTO sources VALUES(?1,?2,'resume.txt','text/plain',?3,?4,NULL,'now')",
        params![
            source,
            name,
            format!("{:x}", Sha256::digest(bytes)),
            bytes.len() as i64
        ],
    )
    .unwrap();
    db.execute_batch("INSERT INTO import_sessions VALUES('session',0); INSERT INTO background_tasks(id,kind,payload,idempotency_key,status,created_at,updated_at) VALUES('task','extract-text','{}','import','completed',0,0);").unwrap();
    let draft = import::classify(std::str::from_utf8(bytes).unwrap(), "Old resume");
    db.execute("INSERT INTO import_items(id,session_id,source_id,task_id,revision,draft,status,created_at) VALUES('item','session',?1,'task',3,?2,'review',0)",params![source,serde_json::to_string(&draft).unwrap()]).unwrap();
    drop(db);
    let mut store = Store::open(dir.path()).unwrap();
    assert_eq!(store.workspace().unwrap().schema_version, 9);
    assert_eq!(store.imports(0).unwrap()[0]["revision"], 3);
    let document = store
        .review_import("item", 3, draft, true)
        .unwrap()
        .unwrap();
    assert_eq!(store.entity_sources(&document).unwrap()[0].id, source);
    assert_eq!(store.source_bytes(&source).unwrap(), bytes);
    let db = Connection::open(dir.path().join("cowworker.db")).unwrap();
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM import_item_sources WHERE item_id='item'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}
#[test]
fn structured_web_vacancy_extracts_without_executing_page_scripts() {
    let html = br#"<html><head><script type="application/ld+json">{"@graph":[{"@type":"JobPosting","title":"Engineer","hiringOrganization":{"name":"Example"},"description":"<p>Build useful software &amp; help users.</p><p>Rust experience.</p>","jobLocationType":"TELECOMMUTE"}]}</script></head><body><nav>Unrelated menu</nav><script>doNotExecute()</script>Loading...</body></html>"#;
    let text = import::formats::extract(html, "text/html").unwrap();
    let draft = import::classify(&text, "Web source");
    assert_eq!(draft.entity_type, "vacancy");
    assert_eq!(draft.company, "Example");
    assert_eq!(draft.title, "Engineer");
    assert_eq!(draft.work_mode, "Remote");
    assert!(
        draft.content.contains("software & help users"),
        "{}",
        draft.content
    );
    assert!(!draft.content.contains("doNotExecute"));
    assert!(!draft.content.contains("Unrelated menu"));
}

#[test]
fn exported_pdf_wraps_long_paragraphs_without_losing_words() {
    let paragraph = "This is a long paragraph about developing reliable desktop applications, maintaining data integrity, improving accessibility, and helping people work with their career documents. ".repeat(30);
    let bytes = import::pdf::export(&paragraph).unwrap();
    let text = import::formats::extract(&bytes, "application/pdf").unwrap();
    assert_eq!(
        text.split_whitespace().collect::<Vec<_>>(),
        paragraph.split_whitespace().collect::<Vec<_>>()
    );
}
#[test]
#[cfg(windows)]
#[ignore = "Requires Windows OCR with an installed language pack"]
fn screenshot_ocr_recognizes_job_text() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/vacancy-screen-1.png");
    let bytes = std::fs::read(path).expect("Run scripts/morning-fixtures.py first");
    let text = import::ocr::extract(&bytes).unwrap();
    println!("{text}");
    assert!(text.contains("Senior Software Engineer"));
    assert!(text.contains("Morning Example"));
}
#[test]
#[cfg(windows)]
#[ignore = "Requires Windows OCR with an installed language pack"]
fn scanned_pdf_ocr_recognizes_text_without_a_text_layer() {
    let bytes = include_bytes!("fixtures/scanned-vacancy.pdf");
    let text = import::formats::extract(bytes, "application/pdf").unwrap();
    assert!(text.contains("Morning Example"));
}
#[test]
fn combined_sources_and_job_documents_keep_originals_and_context() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let first = s
        .import_bytes(
            "first.txt",
            "text/plain",
            b"Title: Engineer\nCompany: Example\nRequirements\nRust",
            None,
        )
        .unwrap();
    let second = s
        .import_bytes(
            "second.txt",
            "text/plain",
            b"Responsibilities\nBuild apps\nSalary: Unknown",
            None,
        )
        .unwrap();
    s.run_next_task("worker").unwrap();
    s.run_next_task("worker").unwrap();
    let combined = s.combine_imports(vec![first, second]).unwrap();
    let rows = s.imports(0).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == combined)
        .unwrap();
    let draft: ImportDraft = serde_json::from_value(row["draft"].clone()).unwrap();
    let vacancy = s.review_import(&combined, 1, draft, true).unwrap().unwrap();
    assert_eq!(s.entity_sources(&vacancy).unwrap().len(), 2);
    let original =
        "Jane Doe\nExperience\nBuilt desktop applications\nEducation\nExample University";
    let resume = s
        .save_document(DocumentInput {
            id: None,
            expected_revision: None,
            title: "Original resume".into(),
            kind: "resume".into(),
            content: original.into(),
        })
        .unwrap();
    let workspace = s.workspace().unwrap();
    let version = workspace
        .documents
        .iter()
        .find(|d| d.id == resume)
        .unwrap()
        .versions[0]
        .id
        .clone();
    let copy = s
        .prepare_job_document(&vacancy, 1, &version, "resume")
        .unwrap();
    let letter = s
        .prepare_job_document(&vacancy, 1, &version, "cover-letter")
        .unwrap();
    assert_ne!(copy, resume);
    assert_eq!(s.document_content(&version).unwrap(), original);
    s.configure_provider(
        ProviderConfig {
            endpoint: "http://127.0.0.1:9999/chat/completions".into(),
            provider: "fixture".into(),
            model: "fixture".into(),
            local: true,
            credential_ref: None,
        },
        None,
    )
    .unwrap();
    let current = s.workspace().unwrap();
    let doc = current.documents.iter().find(|d| d.id == letter).unwrap();
    let request = AiRequest {
        operation_type: "cover-letter".into(),
        entity_type: "document".into(),
        entity_id: letter,
        base_revision: 1,
        base_version: Some(doc.versions[0].id.clone()),
        instruction: "Use only my evidence".into(),
    };
    let preview = s.preview_ai(request.clone()).unwrap();
    assert_eq!(preview.context["originalResume"]["content"], original);
    assert_eq!(preview.context["vacancy"]["id"], vacancy);
    s.patch_vacancy(
        &vacancy,
        1,
        serde_json::json!({"title":"Changed role"})
            .as_object()
            .unwrap()
            .clone(),
    )
    .unwrap();
    assert!(s.start_ai(request, &preview.scope, "stale-job").is_err());
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(s.document_content(&version).unwrap(), original);
}
