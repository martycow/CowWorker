use cowworker_core::{
    ai::{AiOutput, ModelProvider, ProviderResponse, Usage, UsageFilter},
    *,
};
use serde_json::json;
fn vacancy(s: &mut Store) -> String {
    s.save_vacancy(serde_json::from_value(json!({"title":"Engineer","company":"Company","location":"","workMode":"Remote","sourceUrl":"","description":"Original evidence","notes":"","status":"saved"})).unwrap()).unwrap()
}
struct Fake {
    cancel: Option<(std::path::PathBuf, String)>,
}
struct LostResponse;
#[test]
fn company_clear_and_merge_respect_explicit_empty_overrides() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let source = s
        .save_company(CompanyInput {
            id: None,
            expected_revision: None,
            name: "Source".into(),
            fields: json!({"industry":"Software"}),
            notes: "Source notes".into(),
        })
        .unwrap();
    let target = s
        .save_company(CompanyInput {
            id: None,
            expected_revision: None,
            name: "Target".into(),
            fields: json!({}),
            notes: "".into(),
        })
        .unwrap();
    assert!(s
        .merge_companies(&source, &target, 1, 1)
        .unwrap_err()
        .contains("explicitly empty"));
    s.save_company(CompanyInput {
        id: Some(source.clone()),
        expected_revision: Some(1),
        name: "Source".into(),
        fields: json!({}),
        notes: "".into(),
    })
    .unwrap();
    let company = s.companies("Source", 0, 25).unwrap().remove(0);
    assert!(company.fields["industry"].is_null());
    s.merge_companies(&source, &target, 2, 1).unwrap();
}
impl ModelProvider for LostResponse {
    fn call(&self, _: &AiPreview) -> Result<ProviderResponse, String> {
        Err("Connection lost after dispatch".into())
    }
}
#[test]
fn unknown_transport_cannot_be_redispatched_through_cancel_retry() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let id = vacancy(&mut s);
    s.configure_provider(
        ProviderConfig {
            endpoint: "http://127.0.0.1:9999/chat/completions".into(),
            provider: "fixture".into(),
            model: "m".into(),
            local: true,
            credential_ref: None,
        },
        None,
    )
    .unwrap();
    let request = AiRequest {
        operation_type: "job-analysis".into(),
        entity_type: "vacancy".into(),
        entity_id: id,
        base_revision: 1,
        base_version: None,
        instruction: "Check".into(),
    };
    let preview = s.preview_ai(request.clone()).unwrap();
    s.start_ai(request, &preview.scope, "lost").unwrap();
    let task = s
        .claim_task("runner", chrono::Utc::now().timestamp())
        .unwrap()
        .unwrap();
    assert!(s.execute_ai(&task, &LostResponse).is_err());
    assert_eq!(s.task_by_id(&task.id).unwrap().status, "waiting");
    assert!(s.retry_task(&task.id).is_err());
    s.cancel_task(&task.id).unwrap();
    assert!(s.retry_task(&task.id).is_err());
    let usage = s
        .ai_usage(UsageFilter {
            limit: 25,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(usage["totals"]["unknownCostAttempts"], 1);
    assert!(usage["totals"]["totalTokens"].is_null());
}

#[test]
fn logo_failures_and_company_label_corrections_preserve_managed_sources() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let id = vacancy(&mut s);
    let company = s.workspace().unwrap().vacancies[0]
        .company_id
        .clone()
        .unwrap();
    let image = image::DynamicImage::new_rgb8(16, 16);
    let mut bytes = std::io::Cursor::new(vec![]);
    image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    s.save_company_logo(&company, 1, "fixture.png", bytes.get_ref())
        .unwrap();
    let original = s.company_logo(&company).unwrap().unwrap();
    assert!(s.save_company_logo(&company, 2, "bad.png", b"bad").is_err());
    assert_eq!(s.company_logo(&company).unwrap().unwrap(), original);
    s.patch_vacancy(
        &id,
        1,
        json!({"company":"Another company"})
            .as_object()
            .unwrap()
            .clone(),
    )
    .unwrap();
    let target = s.workspace().unwrap().vacancies[0]
        .company_id
        .clone()
        .unwrap();
    assert_ne!(target, company);
    s.merge_companies(&company, &target, 2, 1).unwrap();
    assert_eq!(s.company_logo(&target).unwrap().unwrap(), original);
    let backup = dir.path().join("backup");
    s.backup(&backup).unwrap();
    let restored = dir.path().join("restore");
    backup::restore(&backup, &restored).unwrap();
    assert_eq!(
        Store::open(restored)
            .unwrap()
            .company_logo(&target)
            .unwrap()
            .unwrap(),
        original
    );
}
impl ModelProvider for Fake {
    fn call(&self, _: &AiPreview) -> Result<ProviderResponse, String> {
        if let Some((path, id)) = &self.cancel {
            Store::open(path).unwrap().cancel_task(id).unwrap();
        }
        Ok(ProviderResponse {
            output: Ok(AiOutput {
                summary: "Based on supplied evidence".into(),
                fields: Some(
                    json!({"notes":"Reviewed suggestion"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                content: None,
            }),
            usage: Usage {
                input_tokens: Some(10),
                output_tokens: Some(5),
                total_tokens: Some(15),
                actual_micros: Some(125),
                currency: Some("USD".into()),
                quality: "reported".into(),
                ..Default::default()
            },
            request_id: Some("fixture-request".into()),
        })
    }
}
#[test]
fn ai_authorization_attempt_accounting_and_cancellation() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let id = vacancy(&mut s);
    s.configure_provider(
        ProviderConfig {
            endpoint: "http://127.0.0.1:9999/v1/chat/completions".into(),
            provider: "Fixture".into(),
            model: "fixture".into(),
            local: true,
            credential_ref: None,
        },
        None,
    )
    .unwrap();
    let req = AiRequest {
        operation_type: "job-analysis".into(),
        entity_type: "vacancy".into(),
        entity_id: id.clone(),
        base_revision: 1,
        base_version: None,
        instruction: "Inspect evidence".into(),
    };
    let preview = s.preview_ai(req.clone()).unwrap();
    assert!(s.start_ai(req.clone(), "wrong scope", "bad").is_err());
    let op = s.start_ai(req.clone(), &preview.scope, "first").unwrap();
    assert_eq!(
        s.start_ai(req.clone(), &preview.scope, "first").unwrap(),
        op
    );
    let task = s
        .claim_task("runner", chrono::Utc::now().timestamp())
        .unwrap()
        .unwrap();
    let result = s.execute_ai(&task, &Fake { cancel: None }).unwrap();
    s.finish_task(&task, "runner", Ok(result)).unwrap();
    assert!(s.execute_ai(&task, &Fake { cancel: None }).is_err());
    let proposal = s.propose_ai_output(&op).unwrap();
    assert_eq!(s.propose_ai_output(&op).unwrap(), proposal);
    assert!(s.review_proposal(&proposal, true, false).is_err());
    s.review_proposal(&proposal, true, true).unwrap();
    assert_eq!(s.propose_ai_output(&op).unwrap(), proposal);
    let mut req = req;
    req.base_revision = 2;
    let preview = s.preview_ai(req.clone()).unwrap();
    s.start_ai(req, &preview.scope, "second").unwrap();
    let task = s
        .claim_task("runner", chrono::Utc::now().timestamp())
        .unwrap()
        .unwrap();
    assert!(s
        .execute_ai(
            &task,
            &Fake {
                cancel: Some((dir.path().into(), task.id.clone()))
            }
        )
        .is_err());
    let usage = s
        .ai_usage(UsageFilter {
            limit: 25,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(usage["totals"]["operations"], 2);
    assert_eq!(usage["totals"]["attempts"], 2);
    assert_eq!(usage["totals"]["totalTokens"], 30);
    assert_eq!(usage["costs"][0]["actualMicros"], 250);
    assert_eq!(
        s.workspace().unwrap().vacancies[0].notes,
        "Reviewed suggestion"
    );
}
#[test]
fn persistent_import_review_is_atomic_idempotent_and_preserves_original() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let original="Title: Engineer\r\nCompany: Exact Company\r\nRequirements\r\nRust\r\nResponsibilities\r\nBuild software\r\nSalary: unknown currency";
    let item = s
        .import_bytes("job.txt", "text/plain", original.as_bytes(), None)
        .unwrap();
    s.run_next_task("runner").unwrap();
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    let row = s.imports(0).unwrap()[0].clone();
    let mut draft: ImportDraft = serde_json::from_value(row["draft"].clone()).unwrap();
    assert_eq!(draft.entity_type, "vacancy");
    assert_eq!(draft.structured["salaryRaw"], "unknown currency");
    assert!(draft.structured.get("currency").is_none());
    draft.title = "Corrected role".into();
    s.review_import(&item, 1, draft.clone(), false).unwrap();
    assert!(s.review_import(&item, 1, draft.clone(), true).is_err());
    let target = s
        .review_import(&item, 2, draft.clone(), true)
        .unwrap()
        .unwrap();
    assert_eq!(
        s.review_import(&item, 2, draft, true).unwrap(),
        Some(target.clone())
    );
    assert_eq!(s.workspace().unwrap().vacancies.len(), 1);
    assert_eq!(
        s.source_bytes(row["sourceId"].as_str().unwrap()).unwrap(),
        original.as_bytes()
    );
    assert_eq!(s.entity_sources(&target).unwrap().len(), 1);
    let bad = s
        .import_bytes("empty.txt", "text/plain", b"test", None)
        .unwrap();
    s.run_next_task("runner").unwrap();
    let rows = s.imports(0).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == bad)
        .unwrap();
    let mut draft: ImportDraft = serde_json::from_value(row["draft"].clone()).unwrap();
    draft.entity_type = "vacancy".into();
    draft.company = String::new();
    assert!(s.review_import(&bad, 1, draft, true).is_err());
    assert_eq!(s.workspace().unwrap().vacancies.len(), 1);
}
#[test]
fn company_merge_preserves_stages_and_split_is_revision_guarded() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let first = vacancy(&mut s);
    let second = vacancy(&mut s);
    assert_eq!(s.companies("", 0, 25).unwrap().len(), 1);
    let from = s.companies("", 0, 25).unwrap()[0].clone();
    let target = s
        .save_company(CompanyInput {
            id: None,
            expected_revision: None,
            name: "Independent".into(),
            fields: json!({}),
            notes: "".into(),
        })
        .unwrap();
    s.prepare_application(&first).unwrap();
    s.prepare_application(&second).unwrap();
    s.link_company(&second, 1, &target).unwrap();
    assert!(s.link_company(&second, 1, &target).is_err());
    s.merge_companies(&from.id, &target, 1, 1).unwrap();
    let c = &s.companies("", 0, 25).unwrap()[0];
    assert_eq!(c.vacancy_count, 2);
    assert_eq!(c.stages["preparing"], 2);
    assert_eq!(s.workspace().unwrap().vacancies[0].company, "Company");
    s.set_employment(&target, 2, "current", "Engineer", Some("2026-01-01"), None)
        .unwrap();
    assert_eq!(
        s.companies("", 0, 25).unwrap()[0].relationships[0]["kind"],
        "current"
    );
}
#[test]
fn binary_export_round_trip_and_source_limits() {
    let text="CowWorker export verification\n\nResume v1\nТочный текст и опыт работы\n• Rust and TypeScript\nA & B < C\n\nOnly the selected immutable version is exported.";
    let docx = import::formats::export_docx(text).unwrap();
    let extracted = import::formats::extract_docx(&docx).unwrap();
    assert_eq!(extracted.trim_end(), text);
    let pdf = import::pdf::export(text).unwrap();
    let extracted = import::formats::extract(&pdf, "application/pdf").unwrap();
    assert!(extracted.contains("Точный текст и опыт работы"));
    assert!(extracted.contains("Resume v1"));
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../output/verification");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(output.join("export-fixture.pdf"), pdf).unwrap();
    std::fs::write(output.join("export-fixture.docx"), docx).unwrap();
    assert!(import::formats::extract(b"not a pdf", "application/pdf").is_err());
    assert!(import::formats::extract_docx(b"broken zip").is_err());
    assert!(import::formats::extract(&vec![b'x'; 1_000_001], "text/plain").is_err());
    for address in [
        "127.0.0.1",
        "10.1.2.3",
        "169.254.169.254",
        "::1",
        "::ffff:127.0.0.1",
        "fc00::1",
        "198.18.0.1",
    ] {
        assert!(!import::web::public_address(address.parse().unwrap()));
    }
}
