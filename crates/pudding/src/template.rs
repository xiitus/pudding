use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::model::{default_template, Template};
use crate::paths::{states_dir, templates_dir};

pub fn load_template(name: &str) -> Result<Template> {
    let path = template_path(name);
    if path.exists() {
        let data = fs::read_to_string(&path)?;
        let tpl = serde_json::from_str::<Template>(&data)?;
        Ok(tpl)
    } else {
        Ok(default_template())
    }
}

pub fn save_template(template: &Template) -> Result<()> {
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
    let path = state_path(name);
    let data = fs::read_to_string(&path)?;
    let tpl = serde_json::from_str::<Template>(&data)?;
    Ok(tpl)
}

pub fn save_state(name: &str, template: &Template) -> Result<()> {
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
