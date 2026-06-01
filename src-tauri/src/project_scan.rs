use crate::error::{Result, SkillMasterError};
use crate::models::{AppState, Skill, SkillReference, SkillSource, ReferenceScope, ReferenceStatus, ManagedLinks};
use crate::codex_sync::{validate_managed_link, create_directory_link, ManagedLinkValidation};
use crate::skill_library::{read_skill_metadata, copy_dir_all, should_skip_scan_dir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub is_managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedCategory {
    pub name: String,
    pub path: PathBuf,
    pub skills: Vec<ScannedSkill>,
}

#[derive(Debug, Clone)]
pub enum InternalImportResult {
    Success,
    Conflict {
        skill_id: String,
        library_name: String,
        project_name: String,
    },
}

pub fn scan_project_skills(state: &AppState, project_root: &Path) -> Result<Vec<ScannedCategory>> {
    let mut categories = Vec::new();
    scan_project_skills_recursive(state, project_root, project_root, &mut categories)?;
    categories.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(categories)
}

fn scan_project_skills_recursive(
    state: &AppState,
    project_root: &Path,
    current_dir: &Path,
    categories: &mut Vec<ScannedCategory>,
) -> Result<()> {
    if !current_dir.is_dir() {
        return Ok(());
    }

    let skills_dir = current_dir.join("skills");
    if skills_dir.is_dir() {
        let mut scanned_skills = Vec::new();
        for entry in fs::read_dir(&skills_dir)? {
            let entry = entry?;
            let skill_path = entry.path();
            if !skill_path.is_dir() {
                continue;
            }
            if skill_path.join("SKILL.md").exists() {
                if let Ok(metadata) = read_skill_metadata(&skill_path) {
                    let mut is_managed = false;
                    if let Some(matching_skill) = state.skills.iter().find(|s| s.id == metadata.id) {
                        if let Ok(validation) = validate_managed_link(&matching_skill.library_path, &skill_path) {
                            if validation == ManagedLinkValidation::Valid {
                                is_managed = true;
                            }
                        }
                    }
                    scanned_skills.push(ScannedSkill {
                        id: metadata.id,
                        name: metadata.name,
                        description: metadata.description,
                        path: skill_path,
                        is_managed,
                    });
                }
            }
        }
        if !scanned_skills.is_empty() {
            scanned_skills.sort_by(|a, b| a.name.cmp(&b.name));
            let rel_path = current_dir.strip_prefix(project_root).unwrap_or(current_dir);
            let cat_name = if rel_path.as_os_str().is_empty() {
                ".".to_string()
            } else {
                rel_path.to_string_lossy().replace('\\', "/")
            };
            categories.push(ScannedCategory {
                name: cat_name,
                path: current_dir.to_path_buf(),
                skills: scanned_skills,
            });
        }
    }

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "skills" {
            continue;
        }
        if should_skip_scan_dir(&name_str) {
            continue;
        }
        scan_project_skills_recursive(state, project_root, &path, categories)?;
    }

    Ok(())
}

pub fn import_project_skill(
    state: &mut AppState,
    project_name: &str,
    skill_path: &Path,
    strategy: Option<&str>,
) -> Result<InternalImportResult> {
    if !skill_path.is_dir() {
        return Err(SkillMasterError::MissingDirectory(skill_path.to_path_buf()));
    }
    let metadata = read_skill_metadata(skill_path)?;
    let skill_id = metadata.id.clone();
    let target_library_path = state.skill_library_path.join(&skill_id);

    let has_conflict = state.skills.iter().any(|s| s.id == skill_id) || target_library_path.exists();
    if has_conflict && strategy.is_none() {
        let library_name = state
            .skills
            .iter()
            .find(|s| s.id == skill_id)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| metadata.name.clone());
        return Ok(InternalImportResult::Conflict {
            skill_id,
            library_name,
            project_name: metadata.name,
        });
    }

    let do_copy = if has_conflict {
        strategy == Some("overwrite")
    } else {
        true
    };

    if do_copy {
        if target_library_path.exists() {
            fs::remove_dir_all(&target_library_path)?;
        }
        copy_dir_all(skill_path, &target_library_path)?;
        if let Some(existing_skill) = state.skills.iter_mut().find(|s| s.id == skill_id) {
            existing_skill.name = metadata.name.clone();
            existing_skill.description = metadata.description.clone();
            existing_skill.library_path = target_library_path.clone();
        } else {
            state.skills.push(Skill {
                id: skill_id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                library_path: target_library_path.clone(),
                source: SkillSource::local(Some(skill_path.to_path_buf())),
                references: Vec::new(),
                managed_links: ManagedLinks::default(),
                conflict: None,
            });
        }
    } else {
        if !state.skills.iter().any(|s| s.id == skill_id) {
            state.skills.push(Skill {
                id: skill_id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                library_path: target_library_path.clone(),
                source: SkillSource::local(Some(skill_path.to_path_buf())),
                references: Vec::new(),
                managed_links: ManagedLinks::default(),
                conflict: None,
            });
        }
    }

    if skill_path.exists() {
        let meta = fs::symlink_metadata(skill_path)?;
        if !meta.file_type().is_symlink() {
            fs::remove_dir_all(skill_path)?;
        } else {
            crate::codex_sync::remove_managed_link(skill_path)?;
        }
    }

    create_directory_link(&target_library_path, skill_path)?;

    if let Some(skill) = state.skills.iter_mut().find(|s| s.id == skill_id) {
        let ref_id = format!(
            "ref-{:x}",
            crate::commands::md5_like_hash(skill_path.to_string_lossy().as_bytes())
        );
        skill.references.retain(|r| r.target_path != skill_path);
        skill.references.push(SkillReference {
            id: ref_id,
            target_name: project_name.to_string(),
            target_path: skill_path.to_path_buf(),
            scope: ReferenceScope::Project,
            status: ReferenceStatus::Healthy,
        });
    }

    state.skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(InternalImportResult::Success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::default_state;
    use tempfile::tempdir;
    use std::fs;

    fn make_skill(path: &Path, id: &str, name: &str, desc: &str) {
        let skill_dir = path.join(id);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: \"{}\"\ndescription: \"{}\"\n---\n", name, desc),
        )
        .unwrap();
    }

    #[test]
    fn test_scan_and_categorize() {
        let dir = tempdir().unwrap();
        let proj_root = dir.path().join("my-project");
        fs::create_dir_all(&proj_root).unwrap();

        // 模块 A (sub-a/skills)
        let sub_a_skills = proj_root.join("sub-a").join("skills");
        fs::create_dir_all(&sub_a_skills).unwrap();
        make_skill(&sub_a_skills, "skill-a1", "Skill A1", "Desc A1");
        make_skill(&sub_a_skills, "skill-a2", "Skill A2", "Desc A2");

        // 模块 B (sub-b/deep/skills)
        let sub_b_skills = proj_root.join("sub-b").join("deep").join("skills");
        fs::create_dir_all(&sub_b_skills).unwrap();
        make_skill(&sub_b_skills, "skill-b1", "Skill B1", "Desc B1");

        // 根目录 (skills)
        let root_skills = proj_root.join("skills");
        fs::create_dir_all(&root_skills).unwrap();
        make_skill(&root_skills, "skill-root", "Skill Root", "Desc Root");

        // 排除的目录 (node_modules/skills)
        let nm_skills = proj_root.join("node_modules").join("skills");
        fs::create_dir_all(&nm_skills).unwrap();
        make_skill(&nm_skills, "skill-nm", "Skill NM", "Desc NM");

        let library_root = dir.path().join("library");
        let mut state = default_state(library_root, None);

        // 模拟已托管的技能：skill-a1 已经是符号链接指向 library
        // 我们在 state 里注册它，并让它是符号链接
        let target_library_path = state.skill_library_path.join("skill-a1");
        fs::create_dir_all(&target_library_path).unwrap();
        fs::write(target_library_path.join("SKILL.md"), "---\nname: \"Skill A1\"\n---\n").unwrap();
        
        // 删除项目中的原生目录，建立符号链接
        let original_a1_path = sub_a_skills.join("skill-a1");
        fs::remove_dir_all(&original_a1_path).unwrap();
        create_directory_link(&target_library_path, &original_a1_path).unwrap();

        state.skills.push(Skill {
            id: "skill-a1".to_string(),
            name: "Skill A1".to_string(),
            description: "Desc A1".to_string(),
            library_path: target_library_path,
            source: SkillSource::default(),
            references: vec![SkillReference {
                id: "ref-1".to_string(),
                target_name: "my-project".to_string(),
                target_path: original_a1_path,
                scope: ReferenceScope::Project,
                status: ReferenceStatus::Healthy,
            }],
            managed_links: ManagedLinks::default(),
            conflict: None,
        });

        let categories = scan_project_skills(&state, &proj_root).unwrap();
        assert_eq!(categories.len(), 3); // ".", "sub-a", "sub-b/deep"

        // 分类顺序按 name 排序
        assert_eq!(categories[0].name, ".");
        assert_eq!(categories[1].name, "sub-a");
        assert_eq!(categories[2].name, "sub-b/deep");

        // 检查 sub-a 里的技能
        let sub_a_cat = &categories[1];
        assert_eq!(sub_a_cat.skills.len(), 2);
        
        let s_a1 = sub_a_cat.skills.iter().find(|s| s.id == "skill-a1").unwrap();
        assert!(s_a1.is_managed);

        let s_a2 = sub_a_cat.skills.iter().find(|s| s.id == "skill-a2").unwrap();
        assert!(!s_a2.is_managed);
    }

    #[test]
    fn test_import_no_conflict() {
        let dir = tempdir().unwrap();
        let library_root = dir.path().join("library");
        let proj_root = dir.path().join("my-project");
        fs::create_dir_all(&proj_root).unwrap();

        let sub_skills = proj_root.join("sub").join("skills");
        fs::create_dir_all(&sub_skills).unwrap();
        make_skill(&sub_skills, "custom-skill", "Custom Skill", "My custom description");

        let mut state = default_state(library_root.clone(), None);
        let skill_path = sub_skills.join("custom-skill");

        let result = import_project_skill(&mut state, "my-project (sub)", &skill_path, None).unwrap();
        
        match result {
            InternalImportResult::Success => {}
            _ => panic!("Expected success"),
        }

        // 验证技能库已存在该技能
        assert!(library_root.join("custom-skill").join("SKILL.md").exists());
        assert_eq!(state.skills.len(), 1);
        assert_eq!(state.skills[0].id, "custom-skill");
        assert_eq!(state.skills[0].references.len(), 1);

        // 验证项目原生目录变成了软链接
        let meta = fs::symlink_metadata(&skill_path).unwrap();
        assert!(meta.file_type().is_symlink());
        let target = fs::read_link(&skill_path).unwrap();
        assert_eq!(target.canonicalize().unwrap(), library_root.join("custom-skill").canonicalize().unwrap());
    }

    #[test]
    fn test_import_with_conflict() {
        let dir = tempdir().unwrap();
        let library_root = dir.path().join("library");
        let proj_root = dir.path().join("my-project");
        fs::create_dir_all(&proj_root).unwrap();

        let sub_skills = proj_root.join("sub").join("skills");
        fs::create_dir_all(&sub_skills).unwrap();
        // 项目里的技能版本，内容是 "Project Version"
        make_skill(&sub_skills, "custom-skill", "Custom Skill Project", "Project Version");

        let mut state = default_state(library_root.clone(), None);
        
        // 模拟技能库里已经有该技能，内容是 "Library Version"
        let lib_skill_path = library_root.join("custom-skill");
        fs::create_dir_all(&lib_skill_path).unwrap();
        fs::write(lib_skill_path.join("SKILL.md"), "---\nname: \"Custom Skill Lib\"\ndescription: \"Library Version\"\n---\n").unwrap();
        state.skills.push(Skill {
            id: "custom-skill".to_string(),
            name: "Custom Skill Lib".to_string(),
            description: "Library Version".to_string(),
            library_path: lib_skill_path.clone(),
            source: SkillSource::default(),
            references: Vec::new(),
            managed_links: ManagedLinks::default(),
            conflict: None,
        });

        let skill_path = sub_skills.join("custom-skill");

        // 1. 无 strategy 导入，应该返回 Conflict
        let result = import_project_skill(&mut state, "my-project (sub)", &skill_path, None).unwrap();
        match result {
            InternalImportResult::Conflict { skill_id, library_name, project_name } => {
                assert_eq!(skill_id, "custom-skill");
                assert_eq!(library_name, "Custom Skill Lib");
                assert_eq!(project_name, "Custom Skill Project");
            }
            _ => panic!("Expected conflict"),
        }

        // 2. 策略为 "keep_existing" (保留技能库已有版本，丢弃项目更改)
        let result_keep = import_project_skill(&mut state, "my-project (sub)", &skill_path, Some("keep_existing")).unwrap();
        match result_keep {
            InternalImportResult::Success => {}
            _ => panic!("Expected success on keep_existing"),
        }

        // 验证库里的内容仍然是 Library Version
        let lib_md_content = fs::read_to_string(lib_skill_path.join("SKILL.md")).unwrap();
        assert!(lib_md_content.contains("Library Version"));

        // 验证项目目录已转换成符号链接且有效
        let meta = fs::symlink_metadata(&skill_path).unwrap();
        assert!(meta.file_type().is_symlink());

        // 3. 重置，策略为 "overwrite" (覆盖技能库版本，使用项目修改)
        // 重新做测试准备
        fs::remove_dir_all(&skill_path).unwrap();
        make_skill(&sub_skills, "custom-skill", "Custom Skill Project", "Project Version");
        state.skills[0].references.clear();
        
        let result_overwrite = import_project_skill(&mut state, "my-project (sub)", &skill_path, Some("overwrite")).unwrap();
        match result_overwrite {
            InternalImportResult::Success => {}
            _ => panic!("Expected success on overwrite"),
        }

        // 验证库里的内容变成了 Project Version
        let lib_md_content = fs::read_to_string(lib_skill_path.join("SKILL.md")).unwrap();
        assert!(lib_md_content.contains("Project Version"));
        assert_eq!(state.skills[0].description, "Project Version");
    }
}
