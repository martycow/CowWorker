#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cowworker_core::*;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

struct AppState {
    store: Mutex<Option<Store>>,
    path: std::path::PathBuf,
}
fn with_store<T>(
    state: State<AppState>,
    action: impl FnOnce(&mut Store) -> Result<T, String>,
) -> Result<T, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Workspace is unavailable. Restart CowWorker.".to_string())?;
    if store.is_none() {
        *store = Some(Store::open(&state.path)?);
    }
    action(store.as_mut().ok_or("Workspace could not be opened.")?)
}
#[tauri::command]
fn load_workspace(state: State<AppState>) -> Result<Workspace, String> {
    with_store(state, |s| s.workspace())
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let path = if cfg!(debug_assertions) {
                std::env::var_os("COWWORKER_DATA_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or(app.path().app_local_data_dir()?.join("dev"))
            } else {
                app.path().app_local_data_dir()?.join("live")
            };
            app.manage(AppState {
                store: Mutex::new(None),
                path: path.clone(),
            });
            tauri::WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
                .data_directory(path.join("webview"))
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_workspace,
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
