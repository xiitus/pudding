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

#[derive(Debug, Clone)]
pub struct Config {
    pub default_command: String,
    // v2では永続設定から外したが、ランタイムとの互換のために保持。
    #[allow(dead_code)]
    pub keybinds: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ConfigFileCompat {
    #[serde(default = "default_command_value")]
    default_command: String,
    #[serde(default)]
    keybinds: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ConfigFileV2<'a> {
    default_command: &'a str,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_file_path();
        if let Ok(data) = fs::read_to_string(&path) {
            return Self::from_json(&data)
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
        let data = self.to_json().map_err(io::Error::other)?;
        write_private_file(&path, &data)
    }

    fn from_json(data: &str) -> serde_json::Result<Self> {
        let raw: ConfigFileCompat = serde_json::from_str(data)?;
        let mut keybinds = default_keybinds();
        keybinds.extend(raw.keybinds);
        Ok(Self {
            default_command: raw.default_command,
            keybinds,
        })
    }

    fn to_json(&self) -> serde_json::Result<String> {
        let raw = ConfigFileV2 {
            default_command: &self.default_command,
        };
        serde_json::to_string_pretty(&raw)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_command: default_command_value(),
            keybinds: default_keybinds(),
        }
    }
}

fn default_command_value() -> String {
    "bash".to_string()
}

fn default_keybinds() -> HashMap<String, String> {
    [
        ("split_vertical", "v"),
        ("split_horizontal", "h"),
        ("resize_left", "H"),
        ("resize_right", "L"),
        ("resize_up", "K"),
        ("resize_down", "J"),
        ("swap_vertical", "S"),
        ("swap_horizontal", "s"),
        ("save_state", "Ctrl+S"),
        ("restore_state", "Ctrl+R"),
        ("focus_next", "Tab"),
        ("quit", "Ctrl+C"),
    ]
    .into_iter()
    .map(|(action, key)| (action.to_string(), key.to_string()))
    .collect()
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

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn from_json_merges_legacy_keybinds_into_defaults() {
        let cfg = Config::from_json(
            r#"{
                "default_command": "zsh",
                "keybinds": {
                    "split_vertical": "Ctrl+V",
                    "quit": "q"
                }
            }"#,
        )
        .expect("legacy config should deserialize");

        assert_eq!(cfg.default_command, "zsh");
        assert_eq!(
            cfg.keybinds.get("split_vertical").map(String::as_str),
            Some("Ctrl+V")
        );
        assert_eq!(cfg.keybinds.get("quit").map(String::as_str), Some("q"));
        assert_eq!(
            cfg.keybinds.get("split_horizontal").map(String::as_str),
            Some("h")
        );
    }

    #[test]
    fn from_json_uses_default_command_when_missing() {
        let cfg = Config::from_json(
            r#"{
                "keybinds": {
                    "quit": "q"
                }
            }"#,
        )
        .expect("config without default_command should deserialize");

        assert_eq!(cfg.default_command, "bash");
        assert_eq!(cfg.keybinds.get("quit").map(String::as_str), Some("q"));
    }

    #[test]
    fn to_json_writes_v2_minimal_shape() {
        let mut cfg = Config {
            default_command: "zsh".to_string(),
            ..Config::default()
        };
        cfg.keybinds.insert("quit".to_string(), "q".to_string());

        let json = cfg.to_json().expect("config should serialize");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("serialized config should be valid JSON");

        assert_eq!(
            value,
            serde_json::json!({
                "default_command": "zsh"
            })
        );
        assert!(!json.contains("keybinds"));
    }
}
