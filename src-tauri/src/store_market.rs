use crate::error::{Result, SkillMasterError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoreSkill {
    pub id: String,
    pub skill_id: String,
    pub name: String,
    pub source: String,
    pub installs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardType {
    AllTime,
    Trending,
    Hot,
}

impl LeaderboardType {
    pub fn from_str(value: &str) -> Self {
        match value {
            "trending" => Self::Trending,
            "hot" => Self::Hot,
            _ => Self::AllTime,
        }
    }

    fn url(&self) -> &'static str {
        match self {
            Self::AllTime => "https://skills.sh/",
            Self::Trending => "https://skills.sh/trending",
            Self::Hot => "https://skills.sh/hot",
        }
    }
}

fn build_http_client(timeout_secs: u64) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("skillmaster")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .unwrap_or_default()
}

pub fn fetch_leaderboard(board: LeaderboardType) -> Result<Vec<StoreSkill>> {
    let html = build_http_client(15)
        .get(board.url())
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| SkillMasterError::CommandFailed(format!("获取 skills.sh 榜单失败：{error}")))?
        .text()
        .map_err(|error| SkillMasterError::CommandFailed(format!("读取 skills.sh 榜单失败：{error}")))?;

    parse_leaderboard_html(&html)
}

pub fn search_skills(query: &str, limit: usize) -> Result<Vec<StoreSkill>> {
    let url = format!(
        "https://skills.sh/api/search?q={}&limit={}",
        urlencoding::encode(query),
        limit
    );
    let body = build_http_client(15)
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| SkillMasterError::CommandFailed(format!("搜索 skills.sh 失败：{error}")))?
        .text()
        .map_err(|error| SkillMasterError::CommandFailed(format!("读取搜索结果失败：{error}")))?;

    parse_search_response(&body)
}

pub fn parse_leaderboard_html(html: &str) -> Result<Vec<StoreSkill>> {
    if let Ok(skills) = parse_next_data(html) {
        if !skills.is_empty() {
            return Ok(skills);
        }
    }

    parse_embedded_skill_objects(html)
}

pub fn parse_search_response(body: &str) -> Result<Vec<StoreSkill>> {
    let value: serde_json::Value = serde_json::from_str(body)?;
    if let Some(array) = value.as_array() {
        return Ok(parse_skills_array(array));
    }

    Ok(value
        .get("skills")
        .and_then(|items| items.as_array())
        .map(|items| parse_skills_array(items))
        .unwrap_or_default())
}

fn parse_next_data(html: &str) -> Result<Vec<StoreSkill>> {
    let marker = r#"<script id="__NEXT_DATA__" type="application/json">"#;
    let start = html
        .find(marker)
        .ok_or_else(|| SkillMasterError::CommandFailed("__NEXT_DATA__ 未找到".to_string()))?
        + marker.len();

    let end = html[start..]
        .find("</script>")
        .ok_or_else(|| SkillMasterError::CommandFailed("skills.sh 页面脚本不完整".to_string()))?
        + start;

    let data: serde_json::Value = serde_json::from_str(&html[start..end])?;
    let items = data
        .pointer("/props/pageProps/initialSkills")
        .or_else(|| data.pointer("/props/pageProps/skills"))
        .or_else(|| data.pointer("/props/pageProps/items"))
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(parse_skills_array(&items))
}

fn parse_embedded_skill_objects(html: &str) -> Result<Vec<StoreSkill>> {
    let pattern = Regex::new(
        r#"(?:\\)?\"source(?:\\)?\":(?:\\)?\"(?P<source>[^"\\]+)(?:\\)?\",(?:[^{}]|\\.)*?(?:(?:\\)?\"skillId(?:\\)?\"|(?:\\)?\"skill_id(?:\\)?\"):(?:\\)?\"(?P<skill_id>[^"\\]+)(?:\\)?\",(?:[^{}]|\\.)*?(?:\\)?\"name(?:\\)?\":(?:\\)?\"(?P<name>[^"\\]*)(?:\\)?\",(?:[^{}]|\\.)*?(?:\\)?\"installs(?:\\)?\":(?P<installs>\d+)"#,
    )
    .map_err(|error| SkillMasterError::CommandFailed(format!("构建 skills.sh 解析规则失败：{error}")))?;

    let fallback_pattern = Regex::new(
        r#"\{"source":"(?P<source>[^"]+)","skill_id":"(?P<skill_id>[^"]+)"(?:,"name":"(?P<name>[^"]*)")?(?:.*?"installs":(?P<installs>\d+))?\}"#,
    )
    .map_err(|error| SkillMasterError::CommandFailed(format!("构建后备解析规则失败：{error}")))?;

    let mut skills = parse_embedded_with_regex(html, &pattern);
    if skills.is_empty() {
        skills = parse_embedded_with_regex(html, &fallback_pattern);
    }
    Ok(skills)
}

fn parse_embedded_with_regex(html: &str, pattern: &Regex) -> Vec<StoreSkill> {
    let mut seen = HashSet::new();
    let mut skills = Vec::new();

    for captures in pattern.captures_iter(html) {
        let source = match captures.name("source") {
            Some(value) => value.as_str().replace(r#"\""#, "\""),
            None => continue,
        };
        let skill_id = match captures.name("skill_id") {
            Some(value) => value.as_str().replace(r#"\""#, "\""),
            None => continue,
        };

        let id = format!("{source}/{skill_id}");
        if !seen.insert(id.clone()) {
            continue;
        }

        let name = captures
            .name("name")
            .map(|value| value.as_str().replace(r#"\""#, "\""))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| skill_id.clone());
        let installs = captures
            .name("installs")
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);

        skills.push(StoreSkill {
            id,
            skill_id,
            name,
            source,
            installs,
        });
    }

    skills
}

fn parse_skills_array(items: &[serde_json::Value]) -> Vec<StoreSkill> {
    let mut seen = HashSet::new();
    let mut skills = Vec::new();

    for item in items {
        let source = item
            .get("source")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let skill_id = item
            .get("skillId")
            .or_else(|| item.get("skill_id"))
            .or_else(|| item.get("id"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();

        if source.is_empty() || skill_id.is_empty() {
            continue;
        }

        let id = format!("{source}/{skill_id}");
        if !seen.insert(id.clone()) {
            continue;
        }

        skills.push(StoreSkill {
            id,
            name: item
                .get("name")
                .and_then(|value| value.as_str())
                .filter(|value| !value.is_empty())
                .unwrap_or(&skill_id)
                .to_string(),
            source,
            skill_id,
            installs: item
                .get("installs")
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
        });
    }

    skills
}
