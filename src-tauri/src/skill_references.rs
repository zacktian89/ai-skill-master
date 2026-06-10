use crate::error::{Result, SkillMasterError};
use crate::managed_link::{
    create_directory_link, managed_link_issue_message, remove_managed_link, validate_managed_link,
    ManagedLinkValidation,
};
use crate::models::{AppState, ReferenceScope, ReferenceStatus, Skill, SkillReference};
use crate::path_utils::reference_id;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSkillReferenceRequest {
    pub skill_id: String,
    pub target_name: String,
    pub root_path: PathBuf,
    pub scope: ReferenceScope,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSkillPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub library_path: PathBuf,
    pub managed_link_targets: Vec<PathBuf>,
    pub affected_projects: Vec<ProjectImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSkillsPreview {
    pub items: Vec<DeleteSkillPreview>,
    pub total_managed_link_targets: usize,
    pub total_affected_projects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectImpact {
    pub project_id: String,
    pub project_name: String,
    pub project_path: PathBuf,
}

pub fn refresh_reference_statuses(skill: &mut Skill) -> Result<()> {
    let source = skill.library_path.clone();
    for reference in &mut skill.references {
        reference.status = match validate_managed_link(&source, &reference.target_path)? {
            ManagedLinkValidation::Valid => ReferenceStatus::Healthy,
            ManagedLinkValidation::Missing => ReferenceStatus::Missing,
            ManagedLinkValidation::MissingSource => ReferenceStatus::Stale,
            ManagedLinkValidation::WrongType | ManagedLinkValidation::WrongTarget { .. } => {
                ReferenceStatus::Conflict
            }
        };
    }
    Ok(())
}

pub fn delete_preview_from_state(state: &AppState, skill_id: &str) -> Result<DeleteSkillPreview> {
    let skill = state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(skill_id.to_string()))?;
    let affected_projects = state
        .projects
        .iter()
        .filter(|project| project.rules.contains_key(skill_id))
        .map(|project| ProjectImpact {
            project_id: project.id.clone(),
            project_name: project.name.clone(),
            project_path: project.path.clone(),
        })
        .collect();

    Ok(DeleteSkillPreview {
        skill_id: skill.id.clone(),
        skill_name: skill.name.clone(),
        library_path: skill.library_path.clone(),
        managed_link_targets: skill
            .references
            .iter()
            .map(|reference| reference.target_path.clone())
            .collect(),
        affected_projects,
    })
}

pub fn delete_previews_from_state(
    state: &AppState,
    skill_ids: &[String],
) -> Result<DeleteSkillsPreview> {
    let mut items = Vec::with_capacity(skill_ids.len());
    for skill_id in skill_ids {
        items.push(delete_preview_from_state(state, skill_id)?);
    }
    Ok(DeleteSkillsPreview {
        total_managed_link_targets: items
            .iter()
            .map(|item| item.managed_link_targets.len())
            .sum(),
        total_affected_projects: items.iter().map(|item| item.affected_projects.len()).sum(),
        items,
    })
}

pub fn add_skill_reference_to_state(
    state: &mut AppState,
    request: AddSkillReferenceRequest,
) -> Result<()> {
    let skill = state
        .skills
        .iter_mut()
        .find(|skill| skill.id == request.skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(request.skill_id.clone()))?;
    let target_path = request.root_path.join(&skill.id);

    if skill
        .references
        .iter()
        .any(|reference| reference.target_path == target_path)
    {
        return Ok(());
    }

    match validate_managed_link(&skill.library_path, &target_path)? {
        ManagedLinkValidation::Valid => {}
        ManagedLinkValidation::Missing => create_directory_link(&skill.library_path, &target_path)?,
        ManagedLinkValidation::WrongTarget { .. } if request.overwrite => {
            remove_managed_link(&target_path)?;
            create_directory_link(&skill.library_path, &target_path)?;
        }
        ManagedLinkValidation::MissingSource => {
            return Err(SkillMasterError::MissingDirectory(
                skill.library_path.clone(),
            ));
        }
        validation => {
            return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                &target_path,
                &validation,
            )));
        }
    }

    skill.references.push(SkillReference {
        id: reference_id(&target_path),
        target_name: request.target_name,
        target_path,
        scope: request.scope,
        status: ReferenceStatus::Healthy,
    });
    Ok(())
}

pub fn remove_skill_reference_from_state(
    state: &mut AppState,
    reference_id: &str,
    remove_external_link: Option<bool>,
) -> Result<()> {
    for skill in &mut state.skills {
        let Some(index) = skill
            .references
            .iter()
            .position(|reference| reference.id == reference_id)
        else {
            continue;
        };
        let reference = skill.references[index].clone();
        for agent in &mut state.agents {
            let target_path = agent.path.join(&skill.id);
            if target_path == reference.target_path {
                agent.rules.remove(&skill.id);
            }
        }
        match validate_managed_link(&skill.library_path, &reference.target_path)? {
            ManagedLinkValidation::Valid => remove_managed_link(&reference.target_path)?,
            ManagedLinkValidation::Missing => {}
            validation => match remove_external_link {
                Some(true) => remove_managed_link(&reference.target_path)?,
                Some(false) => {}
                None => {
                    return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                        &reference.target_path,
                        &validation,
                    )));
                }
            },
        }
        skill.references.remove(index);
        return Ok(());
    }
    Err(SkillMasterError::InvalidPath(format!(
        "找不到引用：{reference_id}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managed_link::create_directory_link;
    use crate::models::{ManagedLinks, Project, ProjectRule, Skill};
    use crate::state_store::default_state;
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    #[test]
    fn delete_preview_includes_managed_links_and_project_impacts() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "Writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-1".to_string(),
                target_name: "Claude".to_string(),
                target_path: dir.path().join("codex").join("writer"),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Healthy,
            }],
            managed_links: ManagedLinks {},
            conflict: None,
        });
        let mut rules = BTreeMap::new();
        rules.insert("writer".to_string(), ProjectRule::Disable);
        state.projects.push(Project {
            id: "p1".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules,
        });

        let preview = delete_preview_from_state(&state, "writer").unwrap();

        assert_eq!(preview.skill_name, "Writer");
        assert_eq!(preview.managed_link_targets.len(), 1);
        assert_eq!(preview.affected_projects.len(), 1);
    }

    #[test]
    fn delete_skills_preview_aggregates_each_selected_skill() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "Writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-writer".to_string(),
                target_name: "Claude".to_string(),
                target_path: dir.path().join("codex").join("writer"),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Healthy,
            }],
            managed_links: ManagedLinks {},
            conflict: None,
        });
        state.skills.push(Skill {
            id: "reviewer".to_string(),
            name: "Reviewer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("reviewer"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: ManagedLinks {},
            conflict: None,
        });
        let mut rules = BTreeMap::new();
        rules.insert("writer".to_string(), ProjectRule::Enable);
        state.projects.push(Project {
            id: "p1".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules,
        });

        let preview =
            delete_previews_from_state(&state, &["writer".to_string(), "reviewer".to_string()])
                .unwrap();

        assert_eq!(preview.items.len(), 2);
        assert_eq!(preview.total_managed_link_targets, 1);
        assert_eq!(preview.total_affected_projects, 1);
        assert_eq!(preview.items[0].skill_id, "writer");
        assert_eq!(preview.items[1].skill_id, "reviewer");
    }

    #[test]
    fn add_reference_keeps_retargeted_link_without_overwrite() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: Default::default(),
            conflict: None,
        });

        let result = add_skill_reference_to_state(
            &mut state,
            AddSkillReferenceRequest {
                skill_id: "html-go".to_string(),
                target_name: "Claude".to_string(),
                root_path: root,
                scope: ReferenceScope::User,
                overwrite: false,
            },
        );

        assert!(result.unwrap_err().to_string().contains("已指向其他位置"));
        assert_eq!(std::fs::read_link(&target).unwrap(), other);
        assert!(state.skills[0].references.is_empty());
    }

    #[test]
    fn add_reference_replaces_retargeted_link_with_overwrite() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: Default::default(),
            conflict: None,
        });

        add_skill_reference_to_state(
            &mut state,
            AddSkillReferenceRequest {
                skill_id: "html-go".to_string(),
                target_name: "Claude".to_string(),
                root_path: root,
                scope: ReferenceScope::User,
                overwrite: true,
            },
        )
        .unwrap();

        assert_eq!(
            std::fs::canonicalize(std::fs::read_link(&target).unwrap()).unwrap(),
            std::fs::canonicalize(library.join("html-go")).unwrap()
        );
        assert_eq!(state.skills[0].references.len(), 1);
        assert_eq!(state.skills[0].references[0].target_path, target);
    }

    #[test]
    fn remove_reference_blocks_on_mismatch_without_force() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-id".to_string(),
                target_name: "Claude".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Conflict,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        let result = remove_skill_reference_from_state(&mut state, "ref-id", None);

        assert!(result.unwrap_err().to_string().contains("已指向其他位置"));
        assert_eq!(std::fs::read_link(&target).unwrap(), other);
        assert_eq!(state.skills[0].references.len(), 1);
    }

    #[test]
    fn remove_reference_handles_mismatch_with_force_true() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-id".to_string(),
                target_name: "Claude".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Conflict,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        remove_skill_reference_from_state(&mut state, "ref-id", Some(true)).unwrap();

        assert!(!target.exists());
        assert!(state.skills[0].references.is_empty());
    }

    #[test]
    fn remove_reference_handles_mismatch_with_force_false() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-id".to_string(),
                target_name: "Claude".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Conflict,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        remove_skill_reference_from_state(&mut state, "ref-id", Some(false)).unwrap();

        assert_eq!(std::fs::read_link(&target).unwrap(), other);
        assert!(state.skills[0].references.is_empty());
    }
}
