use theseus::minecraft_rust::{self as rust, Request, Snapshot};
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("minecraft-rust")
        .invoke_handler(tauri::generate_handler![
            status,
            start,
            cancel_sign_in,
            catalog
        ])
        .build()
}
#[tauri::command]
fn status() -> Result<Snapshot, String> {
    rust::status()
}
#[tauri::command]
async fn start(request: Request) -> Result<Snapshot, String> {
    rust::start(request)
}

#[tauri::command]
fn cancel_sign_in() -> Result<(), String> {
    rust::cancel_sign_in()
}

#[tauri::command]
async fn catalog(profile: String) -> Result<serde_json::Value, String> {
    rust::mods::catalog(&profile).await
}
