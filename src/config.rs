use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workdir: Option<PathBuf>,
    pub last_file: Option<PathBuf>,
    pub edit_mode: bool,
    pub sidebar_visible: bool,
}

fn config_path() -> Option<PathBuf> {
    let base = glib::user_config_dir();
    Some(base.join("glance").join("config.json"))
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return Config::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(cfg: &Config) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(&path, text);
    }
}
