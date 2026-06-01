pub mod app_paths;
pub mod codex_sync;
pub mod commands;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;
pub mod project_scan;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::import_skill,
            commands::preview_import_skills,
            commands::confirm_import_skills,
            commands::preview_delete_skill,
            commands::delete_skill,
            commands::add_skill_reference,
            commands::remove_skill_reference,
            commands::add_project,
            commands::set_project_rule,
            commands::set_current_project,
            commands::reset_project_rules,
            commands::delete_project,
            commands::set_codex_path,
            commands::migrate_library,
            commands::rebuild_state,
            commands::sync_codex,
            commands::read_skill_file,
            commands::scan_project_skills,
            commands::import_project_skill
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SkillMaster");
}
