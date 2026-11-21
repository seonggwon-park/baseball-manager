//! Desktop shell: a window around the interface, with the match session living
//! in this process. The interface reaches it through the single `api` command
//! (see `bm_session::App::call`).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bm_session::{ApiError, App};
use serde_json::Value;
use std::sync::Mutex;
use tauri::State;

struct Engine(Mutex<App>);

#[tauri::command]
fn api(cmd: String, payload: Option<Value>, engine: State<'_, Engine>) -> Result<Value, ApiError> {
    // A panic in an earlier call must not lock the user out of starting a new match.
    let mut app = engine.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    app.call(&cmd, payload.unwrap_or(Value::Null))
}

fn main() {
    tauri::Builder::default()
        .manage(Engine(Mutex::new(App::new())))
        .invoke_handler(tauri::generate_handler![api])
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
