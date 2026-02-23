use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

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
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self).unwrap())
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
