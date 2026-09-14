#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cowworker_core::*;
mod extended;
mod recovery;
use extended::*;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

struct AppState {
    store: Mutex<Option<Store>>,
    open_error: Mutex<Option<String>>,
    path: Mutex<std::path::PathBuf>,
    root: std::path::PathBuf,
    runner_gate: Mutex<()>,
}
#[tauri::command]
fn list_companies(
    state: State<AppState>,
    query: String,
    offset: i64,
    limit: i64,
) -> Result<Vec<Company>, String> {
    with_store(state, |s| s.companies(&query, offset, limit))
}
#[tauri::command]
fn save_company(state: State<AppState>, input: CompanyInput) -> Result<String, String> {
    with_store(state, |s| s.save_company(input))
}
#[tauri::command]
fn set_employment(
    state: State<AppState>,
    company_id: String,
    expected_revision: i64,
    kind: String,
    role: String,
    start: Option<String>,
    end: Option<String>,
) -> Result<String, String> {
    with_store(state, |s| {
        s.set_employment(
            &company_id,
            expected_revision,
            &kind,
            &role,
            start.as_deref(),
            end.as_deref(),
        )
    })
}
#[tauri::command]
fn link_company(
    state: State<AppState>,
    vacancy_id: String,
    expected_revision: i64,
    company_id: String,
) -> Result<(), String> {
    with_store(state, |s| {
        s.link_company(&vacancy_id, expected_revision, &company_id)
    })
}
#[tauri::command]
fn merge_companies(
    state: State<AppState>,
    source: String,
    target: String,
    source_revision: i64,
    target_revision: i64,
) -> Result<(), String> {
    with_store(state, |s| {
        s.merge_companies(&source, &target, source_revision, target_revision)
    })
}
fn with_store<T>(
    state: State<AppState>,
    action: impl FnOnce(&mut Store) -> Result<T, String>,
) -> Result<T, String> {
    if let Some(error) = state.open_error.lock().map_err(|e| e.to_string())?.clone() {
        return Err(error);
    }
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Workspace is unavailable. Restart CowWorker.".to_string())?;
    if store.is_none() {
        *store = Some(Store::open(
            state.path.lock().map_err(|e| e.to_string())?.clone(),
        )?);
    }
    action(store.as_mut().ok_or("Workspace could not be opened.")?)
}
#[tauri::command]
fn load_workspace(state: State<AppState>) -> Result<Workspace, String> {
    with_store(state, |s| s.workspace())
}
#[tauri::command]
fn list_tasks(
    state: State<AppState>,
    offset: i64,
    limit: i64,
) -> Result<Vec<BackgroundTask>, String> {
    with_store(state, |s| s.tasks(offset, limit))
}
#[tauri::command]
fn enqueue_backup(state: State<AppState>) -> Result<String, String> {
    with_store(state, |s| {
        s.enqueue_task(
            "backup",
            serde_json::json!({}),
            &format!(
                "backup-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos()
            ),
            None,
        )
    })
}
#[tauri::command]
fn cancel_task(state: State<AppState>, id: String) -> Result<(), String> {
    with_store(state, |s| s.cancel_task(&id))
}
#[tauri::command]
fn retry_task(state: State<AppState>, id: String) -> Result<(), String> {
    with_store(state, |s| s.retry_task(&id))
}
#[tauri::command]
fn list_proposals(state: State<AppState>, entity_id: String) -> Result<Vec<Proposal>, String> {
    with_store(state, |s| s.proposals(&entity_id))
}
#[tauri::command]
fn review_proposal(
    state: State<AppState>,
    id: String,
    accept: bool,
    replace_overrides: bool,
) -> Result<(), String> {
    with_store(state, |s| s.review_proposal(&id, accept, replace_overrides))
}
#[tauri::command]
fn list_sources(state: State<AppState>, entity_id: String) -> Result<Vec<SourceAsset>, String> {
    with_store(state, |s| s.entity_sources(&entity_id))
}
#[tauri::command]
fn correct_document(
    state: State<AppState>,
    id: String,
    expected_revision: i64,
    title: String,
    kind: String,
) -> Result<(), String> {
    with_store(state, |s| {
        s.correct_document(&id, expected_revision, &title, &kind)
    })
}
#[tauri::command]
fn save_vacancy(state: State<AppState>, input: VacancyInput) -> Result<String, String> {
    with_store(state, |s| s.save_vacancy(input))
}
#[tauri::command]
fn save_document(state: State<AppState>, input: DocumentInput) -> Result<String, String> {
    with_store(state, |s| s.save_document(input))
}
#[tauri::command]
fn document_content(state: State<AppState>, version_id: String) -> Result<String, String> {
    with_store(state, |s| s.document_content(&version_id))
}
#[tauri::command]
fn patch_vacancy(
    state: State<AppState>,
    vacancy_id: String,
    expected_revision: i64,
    fields: serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    with_store(state, |s| {
        s.patch_vacancy(&vacancy_id, expected_revision, fields)
    })
}
#[tauri::command]
fn prepare_application(state: State<AppState>, vacancy_id: String) -> Result<String, String> {
    with_store(state, |s| s.prepare_application(&vacancy_id))
}
#[tauri::command]
fn save_application(state: State<AppState>, input: ApplicationInput) -> Result<(), String> {
    with_store(state, |s| s.save_application(input))
}
#[tauri::command]
fn save_profile(state: State<AppState>, profile: Profile) -> Result<(), String> {
    with_store(state, |s| s.save_profile(profile))
}

#[tauri::command]
fn open_source(
    app: tauri::AppHandle,
    state: State<AppState>,
    vacancy_id: String,
) -> Result<(), String> {
    let source = with_store(state, |s| {
        s.workspace()?
            .vacancies
            .into_iter()
            .find(|v| v.id == vacancy_id)
            .map(|v| v.source_url)
            .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
            .ok_or_else(|| "This vacancy has no valid source URL.".to_string())
    })?;
    app.opener()
        .open_url(source, None::<&str>)
        .map_err(|e| e.to_string())
}

fn main() {
    if let Some(result) = import::process::cli() {
        if result.is_err() {
            std::process::exit(1);
        }
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                let handle = window.app_handle().clone();
                let paths = paths.clone();
                std::thread::spawn(move || {
                    let result = extended::import_paths(handle.state::<AppState>(), paths);
                    let _ = handle.emit("imports-added", result);
                });
            }
        })
        .setup(|app| {
            let path = if cfg!(debug_assertions) {
                std::env::var_os("COWWORKER_DATA_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or(app.path().app_local_data_dir()?.join("dev"))
            } else {
                app.path().app_local_data_dir()?.join("live")
            };
            let active_path = recovery::active_path(&path)?;
            // Finish backup and migration before the runner and UI can open this database.
            let (initial_store, open_error) = match Store::open(&active_path) {
                Ok(store) => (Some(store), None),
                Err(error) => (None, Some(error)),
            };
            app.manage(AppState {
                store: Mutex::new(initial_store),
                open_error: Mutex::new(open_error),
                path: Mutex::new(active_path),
                root: path.clone(),
                runner_gate: Mutex::new(()),
            });
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let owner = format!("{}-{:?}", std::process::id(), std::time::SystemTime::now());
                loop {
                    let state = handle.state::<AppState>();
                    let gate = state.runner_gate.lock().expect("Task runner lock");
                    if state
                        .open_error
                        .lock()
                        .expect("Workspace error lock")
                        .is_some()
                    {
                        drop(gate);
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    }
                    let runner_path = state.path.lock().expect("Workspace path lock").clone();
                    match Store::open(&runner_path) {
                        Ok(mut store) => {
                            if let Ok(executable) = std::env::current_exe() {
                                store.use_extractor(executable);
                            }
                            match store.run_next_task(&owner) {
                                Ok(true) => {
                                    let _ = handle.emit("tasks-changed", ());
                                }
                                Ok(false) => (),
                                Err(error) => {
                                    let _ = handle.emit("task-runner-error", error);
                                }
                            }
                        }
                        Err(error) => {
                            let _ = handle.emit("task-runner-error", error);
                        }
                    }
                    drop(gate);
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });
            tauri::WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
                .data_directory(path.join("webview"))
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_companies,
            save_company,
            set_employment,
            link_company,
            merge_companies,
            load_workspace,
            recovery::choose_backup,
            recovery::restore_workspace,
            provider_config,
            discover_models,
            configure_provider,
            preview_ai,
            start_ai,
            ai_usage,
            ai_operations,
            propose_ai_output,
            accept_document_proposal,
            list_imports,
            import_text,
            import_upload,
            prepare_job_document,
            combine_imports,
            review_import,
            import_files,
            export_document,
            source_bytes,
            company_logo,
            choose_company_logo,
            start_company_research,
            company_research_history,
            list_tasks,
            enqueue_backup,
            cancel_task,
            retry_task,
            list_proposals,
            review_proposal,
            list_sources,
            correct_document,
            document_content,
            patch_vacancy,
            save_vacancy,
            save_document,
            prepare_application,
            save_application,
            save_profile,
            open_source
        ])
        .run(tauri::generate_context!())
        .expect("CowWorker could not start");
}
