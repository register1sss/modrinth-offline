use theseus::minecraft_rust::{self as rust, Request, Snapshot};
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("minecraft-rust")
        .invoke_handler(tauri::generate_handler![status, start, cancel_sign_in])
        .build()
}
#[tauri::command]
fn status() -> Result<Snapshot, String> {
    rust::status()
}
#[tauri::command]
fn start(request: Request) -> Result<Snapshot, String> {
    rust::start(request)
}

#[tauri::command]
fn cancel_sign_in() -> Result<(), String> {
    rust::cancel_sign_in()
}
