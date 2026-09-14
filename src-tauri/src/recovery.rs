use crate::{AppState, Store};
use cowworker_core::backup;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tauri::{Emitter, Manager};

pub fn active_path(root: &Path) -> Result<PathBuf, String> {
    let pointer = root.join("active-workspace.json");
    if !pointer.exists() {
        return Ok(root.into());
    }
    let name: String = serde_json::from_slice(&fs::read(pointer).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if uuid::Uuid::parse_str(&name).is_err() || name.len() != 36 {
        return Err("Invalid restored workspace pointer.".into());
    }
    let path = root.join("restored").join(name);
    if !path.join("cowworker.db").is_file() {
        return Err(
            "Restored workspace is missing. The original workspace has been preserved.".into(),
        );
    }
    Ok(path)
}

#[tauri::command]
pub async fn choose_backup() -> Result<Option<serde_json::Value>, String> {
    let Some(folder) = rfd::AsyncFileDialog::new()
        .set_title("Choose a CowWorker backup folder")
        .pick_folder()
        .await
    else {
        return Ok(None);
    };
    let directory = folder.path().to_path_buf();
    tauri::async_runtime::spawn_blocking(move || {
        let manifest = backup::verify(&directory)?;
        Ok(Some(
            serde_json::json!({"directory":directory,"manifest":manifest}),
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn restore_workspace(app: tauri::AppHandle, directory: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        // The runner must drop its connection before the active workspace changes.
        let _gate = state.runner_gate.lock().map_err(|e| e.to_string())?;
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        let mut path = state.path.lock().map_err(|e| e.to_string())?;
        let name = uuid::Uuid::new_v4().to_string();
        let destination = state.root.join("restored").join(&name);
        backup::restore(Path::new(&directory), &destination)?;
        let replacement = Store::open(&destination)?;
        let pointer = state.root.join("active-workspace.pending");
        let mut file = fs::File::create(&pointer).map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec(&name).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        fs::rename(pointer, state.root.join("active-workspace.json")).map_err(|e| e.to_string())?;
        *store = Some(replacement);
        *path = destination.clone();
        *state.open_error.lock().map_err(|e| e.to_string())? = None;
        let _ = app.emit("workspace-restored", ());
        Ok(destination.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| e.to_string())?
}
