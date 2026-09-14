use crate::{with_store, AppState};
use cowworker_core::*;
use serde_json::Value;
use tauri::Manager;
#[tauri::command]
pub fn prepare_job_document(
    state: State<AppState>,
    vacancy_id: String,
    expected_revision: i64,
    resume_version_id: String,
    kind: String,
) -> Result<String, String> {
    with_store(state, |s| {
        s.prepare_job_document(&vacancy_id, expected_revision, &resume_version_id, &kind)
    })
}
#[tauri::command]
pub fn import_upload(
    state: State<AppState>,
    name: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let media = match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return Err("Choose PDF, DOCX, TXT, Markdown, PNG, JPEG, or WebP.".into()),
    };
    with_store(state, |s| s.import_bytes(&name, media, &bytes, None))
}
#[tauri::command]
pub fn combine_imports(state: State<AppState>, items: Vec<String>) -> Result<String, String> {
    with_store(state, |s| s.combine_imports(items))
}
#[tauri::command]
pub async fn discover_models(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let config = with_store(app.state::<AppState>(), |s| s.provider_config())?
        .ok_or("Save AI settings before loading available model IDs.")?;
    tauri::async_runtime::spawn_blocking(move || cowworker_core::ai::discover_models(config))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn source_bytes(state: State<AppState>, id: String) -> Result<Vec<u8>, String> {
    with_store(state, |s| s.source_bytes(&id))
}
#[tauri::command]
pub fn company_logo(state: State<AppState>, company_id: String) -> Result<Option<Vec<u8>>, String> {
    with_store(state, |s| s.company_logo(&company_id))
}
#[tauri::command]
pub async fn choose_company_logo(
    state: State<'_, AppState>,
    company_id: String,
    expected_revision: i64,
) -> Result<(), String> {
    if let Some(file) = rfd::AsyncFileDialog::new()
        .add_filter("Company logo", &["png", "jpg", "jpeg", "webp", "ico"])
        .pick_file()
        .await
    {
        if std::fs::metadata(file.path())
            .map_err(|e| e.to_string())?
            .len()
            > 2_000_000
        {
            return Err("Company logos must be smaller than 2 MB.".into());
        }
        let bytes = file.read().await;
        with_store(state, |s| {
            s.save_company_logo(&company_id, expected_revision, &file.file_name(), &bytes)
        })?;
    }
    Ok(())
}
#[tauri::command]
pub fn start_company_research(
    state: State<AppState>,
    company_id: String,
    expected_revision: i64,
    url: String,
) -> Result<String, String> {
    with_store(state, |s| {
        s.start_company_research(&company_id, expected_revision, &url)
    })
}
#[tauri::command]
pub fn company_research_history(
    state: State<AppState>,
    company_id: String,
) -> Result<Value, String> {
    with_store(state, |s| s.company_research_history(&company_id))
}
use tauri::State;
#[tauri::command]
pub fn provider_config(state: State<AppState>) -> Result<Option<ProviderConfig>, String> {
    with_store(state, |s| s.provider_config())
}
#[tauri::command]
pub fn configure_provider(
    state: State<AppState>,
    config: ProviderConfig,
    secret: Option<String>,
) -> Result<(), String> {
    with_store(state, |s| s.configure_provider(config, secret.as_deref()))
}
#[tauri::command]
pub fn preview_ai(state: State<AppState>, request: AiRequest) -> Result<AiPreview, String> {
    with_store(state, |s| s.preview_ai(request))
}
#[tauri::command]
pub fn start_ai(
    state: State<AppState>,
    request: AiRequest,
    scope: String,
    key: String,
) -> Result<String, String> {
    with_store(state, |s| s.start_ai(request, &scope, &key))
}
#[tauri::command]
pub fn ai_usage(state: State<AppState>, filter: ai::UsageFilter) -> Result<Value, String> {
    with_store(state, |s| s.ai_usage(filter))
}
#[tauri::command]
pub fn ai_operations(state: State<AppState>, entity_id: String) -> Result<Value, String> {
    with_store(state, |s| s.ai_operations(&entity_id))
}
#[tauri::command]
pub fn propose_ai_output(state: State<AppState>, id: String) -> Result<String, String> {
    with_store(state, |s| s.propose_ai_output(&id))
}
#[tauri::command]
pub fn accept_document_proposal(state: State<AppState>, id: String) -> Result<String, String> {
    with_store(state, |s| s.accept_document_proposal(&id))
}
#[tauri::command]
pub fn list_imports(state: State<AppState>, offset: i64) -> Result<Value, String> {
    with_store(state, |s| s.imports(offset))
}
#[tauri::command]
pub fn import_text(
    state: State<AppState>,
    name: String,
    text: String,
    is_url: bool,
) -> Result<String, String> {
    if text.len() > 1_000_000 {
        return Err("Pasted input exceeds 1 MB.".into());
    }
    with_store(state, |s| {
        s.import_bytes(
            &name,
            if is_url {
                "text/uri-list"
            } else {
                "text/plain"
            },
            text.as_bytes(),
            if is_url { Some(text.trim()) } else { None },
        )
    })
}
#[tauri::command]
pub fn review_import(
    state: State<AppState>,
    id: String,
    expected_revision: i64,
    draft: ImportDraft,
    save: bool,
) -> Result<Option<String>, String> {
    with_store(state, |s| {
        s.review_import(&id, expected_revision, draft, save)
    })
}
pub fn import_paths(
    state: State<AppState>,
    paths: Vec<std::path::PathBuf>,
) -> Result<Vec<Value>, String> {
    let mut results = vec![];
    for path in paths.into_iter().take(100) {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported file")
            .to_string();
        let result = (|| {
            let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
            if !metadata.is_file() || metadata.len() > 25_000_000 {
                return Err("Choose a file up to 25 MB.".into());
            }
            let media = match path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase()
                .as_str()
            {
                "pdf" => "application/pdf",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "txt" => "text/plain",
                "md" => "text/markdown",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                _ => "application/octet-stream",
            };
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            with_store(state.clone(), |s| {
                s.import_bytes(&name, media, &bytes, None)
            })
        })();
        results.push(match result {
            Ok(id) => serde_json::json!({"name":name,"id":id}),
            Err(error) => serde_json::json!({"name":name,"error":error}),
        });
    }
    Ok(results)
}
#[tauri::command]
pub async fn import_files(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let files = rfd::AsyncFileDialog::new()
        .add_filter(
            "Career documents",
            &["txt", "md", "pdf", "docx", "png", "jpg", "jpeg", "webp"],
        )
        .pick_files()
        .await;
    import_paths(
        state,
        files
            .unwrap_or_default()
            .into_iter()
            .map(|f| f.path().to_path_buf())
            .collect(),
    )
}
#[tauri::command]
pub fn export_document(
    state: State<AppState>,
    version_id: String,
    format: String,
) -> Result<Vec<u8>, String> {
    let text = with_store(state, |s| s.document_content(&version_id))?;
    match format.as_str() {
        "pdf" => import::pdf::export(&text),
        "docx" => import::formats::export_docx(&text),
        _ => Err("Choose PDF or DOCX export.".into()),
    }
}
