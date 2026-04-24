use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkill {
    pub name: String,
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub trigger_pattern: String,
    #[serde(default)]
    pub default_prompt: String,
    #[serde(default)]
    pub steps: Vec<String>,
}

fn skills_root() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".neph")
        .join("skills")
}

pub fn list_skill_names() -> Result<Vec<String>> {
    let root = skills_root();
    std::fs::create_dir_all(&root)?;
    let mut names = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let parsed: LocalSkill = serde_yaml::from_str(&content)?;
        validate_skill(&parsed)?;
        names.push(parsed.name);
    }
    names.sort();
    Ok(names)
}

pub fn load_skill(name: &str) -> Result<LocalSkill> {
    let root = skills_root();
    std::fs::create_dir_all(&root)?;
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let parsed: LocalSkill = serde_yaml::from_str(&content)?;
        validate_skill(&parsed)?;
        if parsed.name.eq_ignore_ascii_case(name) {
            return Ok(parsed);
        }
    }
    anyhow::bail!("skill not found: {name}")
}

fn validate_skill(skill: &LocalSkill) -> Result<()> {
    if skill.name.trim().is_empty() {
        anyhow::bail!("skill name cannot be empty");
    }
    if skill.steps.len() > 20 {
        anyhow::bail!("skill has too many steps");
    }
    Ok(())
}
