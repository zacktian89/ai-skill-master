use crate::path_utils::user_home_path;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodexConfig {
    raw: String,
}

impl CodexConfig {
    pub fn load_default() -> Self {
        fs::read_to_string(default_codex_config_path())
            .map(|raw| Self { raw })
            .unwrap_or_default()
    }

    pub fn plugin_enabled(&self, plugin_key: &str) -> bool {
        let header = plugin_header(plugin_key);
        if let Some((start, end)) = section_bounds(&self.raw, &header) {
            read_enabled(&self.raw[start..end]).unwrap_or(true)
        } else {
            true
        }
    }

    pub fn skill_enabled(&self, skill_name: &str) -> bool {
        for block in skill_config_blocks(&self.raw) {
            if read_string_value(block, "name").as_deref() == Some(skill_name) {
                return read_enabled(block).unwrap_or(true);
            }
        }
        true
    }
}

pub fn default_codex_config_path() -> PathBuf {
    user_home_path().join(".codex").join("config.toml")
}

pub fn set_plugin_enabled_default(plugin_key: &str, enabled: bool) -> std::io::Result<()> {
    set_plugin_enabled(&default_codex_config_path(), plugin_key, enabled)
}

pub fn set_skill_enabled_default(skill_name: &str, enabled: bool) -> std::io::Result<()> {
    set_skill_enabled(&default_codex_config_path(), skill_name, enabled)
}

pub fn set_plugin_enabled(path: &Path, plugin_key: &str, enabled: bool) -> std::io::Result<()> {
    let raw = fs::read_to_string(path).unwrap_or_default();
    let header = plugin_header(plugin_key);
    let next = upsert_enabled_in_section(raw, &header, enabled);
    write_config(path, next)
}

pub fn set_skill_enabled(path: &Path, skill_name: &str, enabled: bool) -> std::io::Result<()> {
    let raw = fs::read_to_string(path).unwrap_or_default();
    let next = upsert_skill_config(raw, skill_name, enabled);
    write_config(path, next)
}

pub fn plugin_key_from_path(plugin_dir: &Path, plugin_name: &str) -> Option<String> {
    let parts: Vec<String> = plugin_dir
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect();

    for (index, part) in parts.iter().enumerate() {
        if part == "cache" {
            if let Some(marketplace) = parts.get(index + 1) {
                return Some(format!("{plugin_name}@{marketplace}"));
            }
        }
    }
    None
}

fn write_config(path: &Path, content: String) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn plugin_header(plugin_key: &str) -> String {
    format!("[plugins.\"{}\"]", escape_toml_string(plugin_key))
}

fn upsert_enabled_in_section(raw: String, header: &str, enabled: bool) -> String {
    if let Some((start, end)) = section_bounds(&raw, header) {
        let before = &raw[..start];
        let section = &raw[start..end];
        let after = &raw[end..];
        format!("{before}{}{after}", upsert_enabled_line(section, enabled))
    } else {
        append_block(
            raw,
            &format!("{header}\nenabled = {}\n", bool_literal(enabled)),
        )
    }
}

fn upsert_skill_config(raw: String, skill_name: &str, enabled: bool) -> String {
    for (start, end) in skill_config_block_bounds(&raw) {
        let block = &raw[start..end];
        if read_string_value(block, "name").as_deref() == Some(skill_name) {
            let before = &raw[..start];
            let after = &raw[end..];
            return format!("{before}{}{after}", upsert_enabled_line(block, enabled));
        }
    }

    append_block(
        raw,
        &format!(
            "[[skills.config]]\nname = \"{}\"\nenabled = {}\n",
            escape_toml_string(skill_name),
            bool_literal(enabled)
        ),
    )
}

fn append_block(mut raw: String, block: &str) -> String {
    if !raw.is_empty() && !raw.ends_with('\n') {
        raw.push('\n');
    }
    if !raw.is_empty() {
        raw.push('\n');
    }
    raw.push_str(block);
    raw
}

fn section_bounds(raw: &str, header: &str) -> Option<(usize, usize)> {
    let mut found_start = None;
    for (line_start, line) in line_starts(raw) {
        let trimmed = line.trim();
        if found_start.is_none() {
            if trimmed == header {
                found_start = Some(line_start);
            }
            continue;
        }
        if trimmed.starts_with('[') {
            return Some((found_start.unwrap(), line_start));
        }
    }
    found_start.map(|start| (start, raw.len()))
}

fn skill_config_blocks(raw: &str) -> Vec<&str> {
    skill_config_block_bounds(raw)
        .into_iter()
        .map(|(start, end)| &raw[start..end])
        .collect()
}

fn skill_config_block_bounds(raw: &str) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut current_start = None;
    for (line_start, line) in line_starts(raw) {
        let trimmed = line.trim();
        if trimmed == "[[skills.config]]" {
            if let Some(start) = current_start.replace(line_start) {
                blocks.push((start, line_start));
            }
        } else if current_start.is_some() && trimmed.starts_with('[') {
            let start = current_start.take().unwrap();
            blocks.push((start, line_start));
        }
    }
    if let Some(start) = current_start {
        blocks.push((start, raw.len()));
    }
    blocks
}

fn upsert_enabled_line(section: &str, enabled: bool) -> String {
    let mut result = String::new();
    let mut replaced = false;
    for line in section.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        if content.trim_start().starts_with("enabled") && content.contains('=') {
            let indent_len = content.len() - content.trim_start().len();
            result.push_str(&content[..indent_len]);
            result.push_str("enabled = ");
            result.push_str(bool_literal(enabled));
            if line.ends_with("\r\n") {
                result.push_str("\r\n");
            } else if line.ends_with('\n') {
                result.push('\n');
            }
            replaced = true;
        } else {
            result.push_str(line);
        }
    }
    if !replaced {
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str("enabled = ");
        result.push_str(bool_literal(enabled));
        result.push('\n');
    }
    result
}

fn read_enabled(section: &str) -> Option<bool> {
    read_bool_value(section, "enabled")
}

fn read_bool_value(section: &str, key: &str) -> Option<bool> {
    for line in section.lines() {
        let line = without_comment(line);
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() == key {
            return match right.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

fn read_string_value(section: &str, key: &str) -> Option<String> {
    for line in section.lines() {
        let line = without_comment(line);
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() != key {
            continue;
        }
        let value = right.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].replace("\\\"", "\""));
        }
    }
    None
}

fn without_comment(line: &str) -> &str {
    line.split_once('#').map(|(value, _)| value).unwrap_or(line)
}

fn line_starts(raw: &str) -> Vec<(usize, &str)> {
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in raw.split_inclusive('\n') {
        starts.push((offset, line));
        offset += line.len();
    }
    if raw.is_empty() {
        starts.push((0, ""));
    }
    starts
}

fn bool_literal(enabled: bool) -> &'static str {
    if enabled {
        "true"
    } else {
        "false"
    }
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reads_plugin_and_skill_enabled_values() {
        let config = CodexConfig {
            raw: r#"
[plugins."documents@openai-primary-runtime"]
enabled = false

[[skills.config]]
name = "documents:documents"
enabled = false
"#
            .to_string(),
        };

        assert!(!config.plugin_enabled("documents@openai-primary-runtime"));
        assert!(!config.skill_enabled("documents:documents"));
        assert!(config.plugin_enabled("spreadsheets@openai-primary-runtime"));
        assert!(config.skill_enabled("documents:other"));
    }

    #[test]
    fn writes_plugin_enabled_value() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            r#"[plugins."documents@openai-primary-runtime"]
enabled = true
"#,
        )
        .unwrap();

        set_plugin_enabled(&path, "documents@openai-primary-runtime", false).unwrap();
        let raw = fs::read_to_string(path).unwrap();

        assert!(raw.contains("[plugins.\"documents@openai-primary-runtime\"]"));
        assert!(raw.contains("enabled = false"));
    }

    #[test]
    fn appends_missing_plugin_enabled_value() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        set_plugin_enabled(&path, "documents@openai-primary-runtime", false).unwrap();
        let raw = fs::read_to_string(path).unwrap();

        assert!(raw.contains("[plugins.\"documents@openai-primary-runtime\"]"));
        assert!(raw.contains("enabled = false"));
    }

    #[test]
    fn writes_skill_enabled_value() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            r#"[[skills.config]]
name = "documents:documents"
enabled = true
"#,
        )
        .unwrap();

        set_skill_enabled(&path, "documents:documents", false).unwrap();
        let raw = fs::read_to_string(path).unwrap();

        assert!(raw.contains("name = \"documents:documents\""));
        assert!(raw.contains("enabled = false"));
    }

    #[test]
    fn derives_marketplace_plugin_key_from_cache_path() {
        let key = plugin_key_from_path(
            Path::new("C:/Users/me/.codex/plugins/cache/openai-primary-runtime/documents/1"),
            "documents",
        );

        assert_eq!(key.as_deref(), Some("documents@openai-primary-runtime"));
    }
}
