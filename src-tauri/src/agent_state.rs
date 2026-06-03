use crate::error::{Result, SkillMasterError};
use crate::managed_link::{
    create_directory_link, managed_link_issue_message, remove_managed_link, validate_managed_link,
    ManagedLinkValidation,
};
use crate::models::{
    Agent, AppState, ProjectRule, ReferenceScope, ReferenceStatus, SkillReference,
};
use crate::path_utils::{id_from_path, reference_id, resolve_path_with_home};
use crate::project_scan::{ScannedCategory, ScannedSkill};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAgentRuleRequest {
    pub agent_id: String,
    pub skill_id: String,
    pub rule: ProjectRule,
}

pub fn add_agent_to_state(state: &mut AppState, name: String, path: PathBuf) {
    let resolved_path = resolve_path_with_home(path);
    let id = id_from_path(&resolved_path);
    if !state.agents.iter().any(|agent| agent.id == id) {
        state.agents.push(Agent {
            id,
            name,
            path: resolved_path,
            rules: BTreeMap::new(),
        });
    }
}

pub fn delete_agent_from_state(state: &mut AppState, agent_id: &str) {
    state.agents.retain(|agent| agent.id != agent_id);
}

pub fn set_agent_rule_in_state(state: &mut AppState, request: SetAgentRuleRequest) -> Result<()> {
    let agent = state
        .agents
        .iter_mut()
        .find(|agent| agent.id == request.agent_id)
        .ok_or_else(|| {
            SkillMasterError::InvalidPath(format!("找不到 Agent：{}", request.agent_id))
        })?;

    let agent_path = agent.path.clone();
    let agent_name = agent.name.clone();
    let target_path = agent_path.join(&request.skill_id);

    if request.rule == ProjectRule::Inherit {
        agent.rules.remove(&request.skill_id);
    } else {
        agent.rules.insert(request.skill_id.clone(), request.rule);
    }

    let skill = state
        .skills
        .iter_mut()
        .find(|skill| skill.id == request.skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(request.skill_id.clone()))?;

    if request.rule == ProjectRule::Inherit {
        match validate_managed_link(&skill.library_path, &target_path)? {
            ManagedLinkValidation::Valid => remove_managed_link(&target_path)?,
            ManagedLinkValidation::Missing => {}
            validation => {
                return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                    &target_path,
                    &validation,
                )));
            }
        }
        skill
            .references
            .retain(|reference| reference.target_path != target_path);
        return Ok(());
    }

    match request.rule {
        ProjectRule::Disable => {
            if target_path.exists() {
                remove_managed_link(&target_path)?;
            }
        }
        ProjectRule::Enable => {
            match validate_managed_link(&skill.library_path, &target_path)? {
                ManagedLinkValidation::Valid => {}
                ManagedLinkValidation::Missing => {
                    create_directory_link(&skill.library_path, &target_path)?;
                }
                ManagedLinkValidation::WrongTarget { .. } => {
                    remove_managed_link(&target_path)?;
                    create_directory_link(&skill.library_path, &target_path)?;
                }
                validation => {
                    return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                        &target_path,
                        &validation,
                    )));
                }
            }
            let ref_id = reference_id(&target_path);
            if !skill
                .references
                .iter()
                .any(|reference| reference.id == ref_id)
            {
                skill.references.push(SkillReference {
                    id: ref_id,
                    target_name: agent_name,
                    target_path,
                    scope: ReferenceScope::User,
                    status: ReferenceStatus::Healthy,
                });
            }
        }
        ProjectRule::Inherit => {}
    }

    Ok(())
}

pub fn scan_agent_skills(state: &AppState, agent_path: PathBuf) -> Result<Vec<ScannedCategory>> {
    let resolved_path = resolve_path_with_home(agent_path);
    if !resolved_path.is_dir() {
        return Ok(Vec::new());
    }
    let scan_dir = if resolved_path.join("skills").is_dir() {
        resolved_path.join("skills")
    } else {
        resolved_path.clone()
    };

    let mut scanned_skills = scan_skill_dirs(state, &scan_dir)?;
    add_state_only_agent_skills(state, &resolved_path, &scan_dir, &mut scanned_skills);

    if scanned_skills.is_empty() {
        Ok(Vec::new())
    } else {
        scanned_skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(vec![ScannedCategory {
            name: ".".to_string(),
            path: scan_dir,
            skills: scanned_skills,
        }])
    }
}

fn scan_skill_dirs(state: &AppState, scan_dir: &std::path::Path) -> Result<Vec<ScannedSkill>> {
    let mut scanned_skills = Vec::new();
    let entries = fs::read_dir(scan_dir)?;
    for entry in entries {
        let entry = entry?;
        let skill_path = entry.path();
        if !skill_path.is_dir() || !skill_path.join("SKILL.md").exists() {
            continue;
        }
        if let Ok(metadata) = crate::skill_library::read_skill_metadata(&skill_path) {
            let is_managed = state
                .skills
                .iter()
                .find(|skill| skill.id == metadata.id)
                .and_then(|matching_skill| {
                    validate_managed_link(&matching_skill.library_path, &skill_path).ok()
                })
                == Some(ManagedLinkValidation::Valid);
            scanned_skills.push(ScannedSkill {
                id: metadata.id,
                name: metadata.name,
                description: metadata.description,
                path: skill_path,
                is_managed,
            });
        }
    }
    Ok(scanned_skills)
}

fn add_state_only_agent_skills(
    state: &AppState,
    resolved_path: &std::path::Path,
    scan_dir: &std::path::Path,
    scanned_skills: &mut Vec<ScannedSkill>,
) {
    let agent_rules_and_refs: Vec<String> = state
        .skills
        .iter()
        .filter(|skill| {
            let has_rule = state
                .agents
                .iter()
                .any(|agent| agent.path == resolved_path && agent.rules.contains_key(&skill.id));
            let has_ref = skill.references.iter().any(|reference| {
                reference.target_path.parent() == Some(scan_dir)
                    || reference.target_path.parent() == Some(resolved_path)
            });
            has_rule || has_ref
        })
        .map(|skill| skill.id.clone())
        .collect();

    for skill_id in agent_rules_and_refs {
        if scanned_skills.iter().any(|skill| skill.id == skill_id) {
            continue;
        }
        if let Some(skill) = state.skills.iter().find(|skill| skill.id == skill_id) {
            scanned_skills.push(ScannedSkill {
                id: skill.id.clone(),
                name: skill.name.clone(),
                description: skill.description.clone(),
                path: scan_dir.join(&skill.id),
                is_managed: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managed_link::create_directory_link;
    use crate::models::Skill;
    use crate::state_store::default_state;
    use tempfile::tempdir;

    #[test]
    fn setting_agent_rule_to_inherit_removes_managed_link_and_reference() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("windsurf").join("skills");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("html-go");
        create_directory_link(&library.join("html-go"), &target).unwrap();

        let mut state = default_state(library.clone());
        let mut rules = BTreeMap::new();
        rules.insert("html-go".to_string(), ProjectRule::Enable);
        state.agents.push(Agent {
            id: "windsurf".to_string(),
            name: "Windsurf".to_string(),
            path: root.clone(),
            rules,
        });
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: reference_id(&target),
                target_name: "Windsurf".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Healthy,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        set_agent_rule_in_state(
            &mut state,
            SetAgentRuleRequest {
                agent_id: "windsurf".to_string(),
                skill_id: "html-go".to_string(),
                rule: ProjectRule::Inherit,
            },
        )
        .unwrap();

        assert!(!target.exists());
        assert!(!state.agents[0].rules.contains_key("html-go"));
        assert!(state.skills[0].references.is_empty());
    }
}
