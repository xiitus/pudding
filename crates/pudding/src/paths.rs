use std::{env, path::PathBuf};

pub fn config_dir() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".config");
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn templates_dir() -> PathBuf {
    config_dir().join("pudding").join("templates")
}
