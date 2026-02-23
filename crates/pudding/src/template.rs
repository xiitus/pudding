use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::model::{default_template, Node, Template};
use crate::paths::{states_dir, templates_dir};

pub fn load_template(name: &str) -> Result<Template> {
    validate_store_name(name)?;
    let path = template_path(name);
    if path.exists() {
        let data = fs::read_to_string(&path)?;
        let tpl = serde_json::from_str::<Template>(&data)?;
        validate_template(&tpl)?;
        Ok(tpl)
    } else {
        Ok(default_template())
    }
}

pub fn save_template(template: &Template) -> Result<()> {
    validate_store_name(&template.name)?;
    validate_template(template)?;
    let path = template_path(&template.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(template)?)?;
    Ok(())
}

pub fn template_path(name: &str) -> PathBuf {
    templates_dir().join(format!("{}.json", name))
}

pub fn load_state(name: &str) -> Result<Template> {
    validate_store_name(name)?;
    let path = state_path(name);
    let data = fs::read_to_string(&path)?;
    let tpl = serde_json::from_str::<Template>(&data)?;
    validate_template(&tpl)?;
    Ok(tpl)
}

pub fn save_state(name: &str, template: &Template) -> Result<()> {
    validate_store_name(name)?;
    validate_template(template)?;
    let path = state_path(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(template)?)?;
    Ok(())
}

pub fn state_path(name: &str) -> PathBuf {
    states_dir().join(format!("{}.json", name))
}

fn validate_store_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        bail!("name must be 1..=64 chars");
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("name supports only [A-Za-z0-9_-]");
    }
    Ok(())
}

fn validate_template(template: &Template) -> Result<()> {
    validate_store_name(&template.name)?;
    let mut ids = HashSet::new();
    validate_node(&template.layout, &mut ids)
}

fn validate_node(node: &Node, ids: &mut HashSet<u64>) -> Result<()> {
    if !ids.insert(node.id()) {
        return Err(anyhow!("node id must be unique"));
    }
    match node {
        Node::Bite { name, command, .. } => {
            if name.trim().is_empty() {
                bail!("bite name must not be empty");
            }
            if command.trim().is_empty() {
                bail!("bite command must not be empty");
            }
            Ok(())
        }
        Node::Spoon {
            ratio,
            first,
            second,
            ..
        } => {
            if !(*ratio > 0.0 && *ratio < 1.0) {
                bail!("spoon ratio must be in (0,1)");
            }
            validate_node(first, ids)?;
            validate_node(second, ids)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Node, Orientation, Template};
    use crate::template::{validate_store_name, validate_template};

    #[test]
    fn reject_path_like_name() {
        assert!(validate_store_name("../evil").is_err());
        assert!(validate_store_name("ok_name-1").is_ok());
    }

    #[test]
    fn reject_invalid_template_ratio() {
        let template = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 1.0,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&template).is_err());
    }
}
