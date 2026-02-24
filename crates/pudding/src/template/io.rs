use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use anyhow::Result;

use crate::model::{default_template, Template};
use crate::paths::{states_dir, templates_dir};
use crate::template::validation::{validate_store_name, validate_template};

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
        ensure_dir_secure(parent)?;
    }
    let data = serde_json::to_string_pretty(template)?;
    write_private_file(&path, &data)?;
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
        ensure_dir_secure(parent)?;
    }
    let data = serde_json::to_string_pretty(template)?;
    write_private_file(&path, &data)?;
    Ok(())
}

pub fn state_path(name: &str) -> PathBuf {
    states_dir().join(format!("{}.json", name))
}

fn ensure_dir_secure(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn write_private_file(path: &Path, content: &str) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;

    file.write_all(content.as_bytes())
}
