use serde_json::Value;
use theseus::assistant as ai;
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("assistant")
        .invoke_handler(tauri::generate_handler![
            install,
            install_status,
            install_cancel,
            preferences,
            set_access,
            status,
            chat,
            profiles,
            files,
            read,
            write_config,
            toggle,
            search,
            prepare,
            build,
            collection
        ])
        .build()
}
#[tauri::command]
async fn preferences() -> ai::Result<ai::Preferences> {
    ai::preferences().await
}
#[tauri::command]
async fn set_access(enabled: bool) -> ai::Result<ai::Preferences> {
    ai::set_access(enabled).await
}
#[tauri::command]
async fn status() -> ai::Result<Value> {
    ai::status().await
}
#[tauri::command]
async fn chat(
    messages: Vec<ai::Message>,
    language: String,
) -> ai::Result<Value> {
    ai::chat(messages, &language).await
}
#[tauri::command]
async fn profiles() -> ai::Result<Value> {
    ai::profiles().await
}
#[tauri::command]
async fn files(
    profile_id: String,
    folder: String,
) -> ai::Result<Vec<ai::Entry>> {
    ai::files(&profile_id, &folder).await
}
#[tauri::command]
async fn read(profile_id: String, path: String) -> ai::Result<String> {
    ai::read(&profile_id, &path).await
}
#[tauri::command]
async fn write_config(
    profile_id: String,
    path: String,
    content: String,
) -> ai::Result<String> {
    ai::write_config(&profile_id, &path, &content).await
}
#[tauri::command]
async fn toggle(profile_id: String, path: String) -> ai::Result<String> {
    ai::toggle(&profile_id, &path).await
}
#[tauri::command]
async fn search(
    query: String,
    game: String,
    loader: String,
    kind: String,
) -> ai::Result<Value> {
    ai::search(&query, &game, &loader, &kind).await
}
#[tauri::command]
async fn prepare(request: ai::BuildRequest) -> ai::Result<Value> {
    ai::prepare(request).await
}
#[tauri::command]
async fn build(token: String) -> ai::Result<Value> {
    ai::build(&token).await
}
#[tauri::command]
async fn collection(
    profile_id: String,
    name: String,
    projects: Vec<String>,
) -> ai::Result<String> {
    ai::collection(&profile_id, &name, projects).await
}

#[tauri::command]
async fn install() -> ai::Result<theseus::assistant_setup::SetupProgress> {
    theseus::assistant_setup::begin()
}
#[tauri::command]
async fn install_status() -> ai::Result<theseus::assistant_setup::SetupProgress>
{
    theseus::assistant_setup::get_progress()
}
#[tauri::command]
async fn install_cancel() -> ai::Result<theseus::assistant_setup::SetupProgress>
{
    theseus::assistant_setup::cancel()
}
