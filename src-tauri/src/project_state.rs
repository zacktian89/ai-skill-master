use crate::error::{Result, SkillMasterError};
use crate::models::{AppState, Project, ProjectRule};
use crate::path_utils::id_from_path;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectRequest {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetProjectRuleRequest {
    pub project_id: String,
    pub skill_id: String,
    pub rule: ProjectRule,
}

pub fn add_project_to_state(state: &mut AppState, request: AddProjectRequest) {
    let id = id_from_path(&request.path);
    if !state.projects.iter().any(|project| project.id == id) {
        state.projects.push(Project {
            id,
            name: request.name,
            path: request.path,
            rules: BTreeMap::new(),
        });
    }
}

pub fn set_project_rule_in_state(
    state: &mut AppState,
    request: SetProjectRuleRequest,
) -> Result<()> {
    let project = state
        .projects
        .iter_mut()
        .find(|project| project.id == request.project_id)
        .ok_or_else(|| SkillMasterError::ProjectNotFound(request.project_id.clone()))?;
    if request.rule == ProjectRule::Inherit {
        project.rules.remove(&request.skill_id);
    } else {
        project.rules.insert(request.skill_id, request.rule);
    }
    Ok(())
}

pub fn set_current_project_in_state(
    state: &mut AppState,
    project_id: Option<String>,
) -> Result<()> {
    if let Some(project_id) = project_id.as_deref() {
        if !state
            .projects
            .iter()
            .any(|project| project.id == project_id)
        {
            return Err(SkillMasterError::ProjectNotFound(project_id.to_string()));
        }
    }
    state.current_project_id = project_id;
    Ok(())
}

pub fn reset_project_rules_in_state(state: &mut AppState, project_id: &str) -> Result<()> {
    let project = state
        .projects
        .iter_mut()
        .find(|project| project.id == project_id)
        .ok_or_else(|| SkillMasterError::ProjectNotFound(project_id.to_string()))?;
    project.rules.clear();
    Ok(())
}

pub fn delete_project_from_state(state: &mut AppState, project_id: &str) -> Result<()> {
    let original_len = state.projects.len();
    state.projects.retain(|project| project.id != project_id);
    if state.projects.len() == original_len {
        return Err(SkillMasterError::ProjectNotFound(project_id.to_string()));
    }
    if state.current_project_id.as_deref() == Some(project_id) {
        state.current_project_id = None;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::default_state;
    use tempfile::tempdir;

    #[test]
    fn deleting_current_project_clears_context() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));
        state.current_project_id = Some("demo".to_string());
        state.projects.push(Project {
            id: "demo".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules: BTreeMap::new(),
        });

        delete_project_from_state(&mut state, "demo").unwrap();

        assert!(state.projects.is_empty());
        assert_eq!(state.current_project_id, None);
    }

    #[test]
    fn resetting_project_rules_clears_only_selected_project() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));

        let mut demo_rules = BTreeMap::new();
        demo_rules.insert("writer".to_string(), ProjectRule::Enable);
        let mut keep_rules = BTreeMap::new();
        keep_rules.insert("reviewer".to_string(), ProjectRule::Disable);

        state.projects.push(Project {
            id: "demo".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules: demo_rules,
        });
        state.projects.push(Project {
            id: "keep".to_string(),
            name: "Keep".to_string(),
            path: dir.path().join("keep"),
            rules: keep_rules,
        });

        reset_project_rules_in_state(&mut state, "demo").unwrap();

        assert!(state.projects[0].rules.is_empty());
        assert_eq!(state.projects[1].rules.len(), 1);
    }
}
