pub mod app_paths;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run SkillMaster");
}
