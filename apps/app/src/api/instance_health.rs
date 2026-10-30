use serde_json::Value;
use theseus::instance_health as health;
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("instance_health")
        .invoke_handler(tauri::generate_handler![
            inventory,
            crash_report,
            repair,
            resources,
            antivirus
        ])
        .build()
}
#[tauri::command]
async fn inventory(profile_id: String) -> health::Result<Vec<health::ModInfo>> {
    health::inventory(&profile_id).await
}
#[tauri::command]
async fn crash_report(
    profile_id: String,
    since_ms: u64,
) -> health::Result<Value> {
    health::crash_report(&profile_id, since_ms).await
}
#[tauri::command]
async fn repair(profile_id: String) -> health::Result<Value> {
    health::repair(&profile_id).await
}
#[tauri::command]
async fn resources(profile_id: String) -> health::Result<Value> {
    health::resources(&profile_id).await
}
#[tauri::command]
async fn antivirus(profile_id: String, path: String) -> health::Result<Value> {
    health::antivirus(&profile_id, &path).await
}
