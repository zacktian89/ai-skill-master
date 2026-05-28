pub mod app_paths;
pub mod codex_sync;
pub mod commands;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::import_skill,
            commands::delete_skill,
            commands::set_default_enabled,
            commands::add_project,
            commands::set_project_rule,
            commands::set_current_project,
            commands::set_codex_path,
            commands::sync_codex
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SkillMaster");
}
