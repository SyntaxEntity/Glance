use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Every field carries `#[serde(default)]` so a config written by an older
/// version — or hand-edited to drop a key — still loads the fields it does
/// have, instead of failing to deserialize and resetting everything.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let cfg = Config {
            workdir: Some(PathBuf::from("/home/u/notes")),
            last_file: Some(PathBuf::from("/home/u/notes/readme.md")),
            edit_mode: true,
            sidebar_visible: false,
        };
        let text = serde_json::to_string_pretty(&cfg).unwrap();
        let back: Config = serde_json::from_str(&text).unwrap();
        assert_eq!(back.workdir, cfg.workdir);
        assert_eq!(back.last_file, cfg.last_file);
        assert_eq!(back.edit_mode, cfg.edit_mode);
        assert_eq!(back.sidebar_visible, cfg.sidebar_visible);
    }

    /// A truncated or hand-edited config must not panic — `load` falls back to
    /// defaults via `unwrap_or_default`.
    #[test]
    fn corrupt_json_falls_back_to_default() {
        let broken = "{ this is not json";
        let cfg: Config = serde_json::from_str(broken).unwrap_or_default();
        assert!(cfg.workdir.is_none());
        assert!(!cfg.edit_mode);
        assert!(!cfg.sidebar_visible);
    }

    /// Missing fields must be tolerated so an older config keeps working.
    #[test]
    fn partial_json_uses_defaults_for_missing_fields() {
        let cfg: Config = serde_json::from_str(r#"{"edit_mode":true}"#).unwrap_or_default();
        assert!(cfg.edit_mode);
        assert!(cfg.workdir.is_none());
        assert!(cfg.last_file.is_none());
    }

    #[test]
    fn config_path_is_under_user_config_dir() {
        let path = config_path().expect("config path should resolve");
        assert!(path.ends_with("glance/config.json"));
        assert!(path.starts_with(glib::user_config_dir()));
    }
}
