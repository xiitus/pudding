use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use crate::paths::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_command: String,
    pub keybinds: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_file_path();
        if let Ok(data) = fs::read_to_string(&path) {
            return serde_json::from_str::<Config>(&data)
                .with_context(|| format!("invalid config file: {}", path.display()));
        }
        let cfg = Config::default();
        cfg.save()
            .with_context(|| format!("failed to write config: {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_file_path();
        if let Some(parent) = path.parent() {
            ensure_dir_secure(parent)?;
        }
        let data = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        write_private_file(&path, &data)
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut keybinds = HashMap::new();
        keybinds.insert("split_vertical".to_string(), "v".to_string());
        keybinds.insert("split_horizontal".to_string(), "h".to_string());
        keybinds.insert("resize_left".to_string(), "H".to_string());
        keybinds.insert("resize_right".to_string(), "L".to_string());
        keybinds.insert("resize_up".to_string(), "K".to_string());
        keybinds.insert("resize_down".to_string(), "J".to_string());
        keybinds.insert("swap_vertical".to_string(), "S".to_string());
        keybinds.insert("swap_horizontal".to_string(), "s".to_string());
        keybinds.insert("save_state".to_string(), "Ctrl+S".to_string());
        keybinds.insert("restore_state".to_string(), "Ctrl+R".to_string());
        keybinds.insert("focus_next".to_string(), "Tab".to_string());
        keybinds.insert("quit".to_string(), "Ctrl+C".to_string());
        Config {
            default_command: "bash".to_string(),
            keybinds,
        }
    }
}

pub fn config_file_path() -> PathBuf {
    config_dir().join("pudding").join("config.json")
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
