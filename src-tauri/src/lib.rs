pub mod app_paths;
pub mod managed_link;
pub mod commands;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;
pub mod project_scan;
pub mod store_market;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::fetch_store_leaderboard,
            commands::search_store_skills,
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
            commands::migrate_library,
            commands::read_skill_file,
            commands::read_skill_file_at_path,
            commands::scan_project_skills,
            commands::import_project_skill,
            commands::delete_unmanaged_skill,
            commands::add_agent,
            commands::delete_agent,
            commands::set_agent_rule,
            commands::scan_agent_skills
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SkillMaster");
}

#[cfg(test)]
mod store_market_tests {
    use super::store_market::{parse_leaderboard_html, parse_search_response};

    #[test]
    fn parses_next_data_leaderboard_payload() {
        let html = r#"
        <html>
          <script id="__NEXT_DATA__" type="application/json">
            {"props":{"pageProps":{"initialSkills":[{"source":"openai/skills","skillId":"playwright","name":"Playwright","installs":4000}]}}}
          </script>
        </html>
        "#;

        let skills = parse_leaderboard_html(html).expect("leaderboard payload should parse");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "openai/skills/playwright");
        assert_eq!(skills[0].name, "Playwright");
    }

    #[test]
    fn parses_search_api_payload() {
        let payload = r#"
        [
          {"source":"acme/skills","skillId":"writer","name":"Writer","installs":82}
        ]
        "#;

        let skills = parse_search_response(payload).expect("search payload should parse");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "acme/skills/writer");
        assert_eq!(skills[0].source, "acme/skills");
    }
}
