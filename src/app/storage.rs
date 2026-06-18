use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;

use super::state::Settings;

pub fn load_settings() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };

    let Ok(contents) = fs::read_to_string(path) else {
        return Settings::default();
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save_settings(settings: &Settings) {
    let Some(path) = settings_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "kemoshumai", "pomoshumai")
        .map(|dirs| dirs.config_dir().join("settings.json"))
}
