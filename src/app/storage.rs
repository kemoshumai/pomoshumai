use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::state::{Phase, Settings, TimerStatus};
use super::timer::TimerState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSnapshot {
    pub phase: Phase,
    pub status: TimerStatus,
    pub remaining_secs: u64,
    pub session_duration_secs: u64,
    pub completed_pomodoros: u32,
    #[serde(default)]
    pub saved_at_unix_secs: u64,
}

impl AppSnapshot {
    pub fn from_timer(timer: &TimerState, completed_pomodoros: u32) -> Self {
        Self {
            phase: timer.phase,
            status: timer.status,
            remaining_secs: timer.remaining.as_secs(),
            session_duration_secs: timer.session_duration.as_secs(),
            completed_pomodoros,
            saved_at_unix_secs: unix_now(),
        }
    }
}

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

pub fn load_snapshot() -> Option<AppSnapshot> {
    let path = state_path()?;
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

pub fn save_snapshot(snapshot: &AppSnapshot) {
    let Some(path) = state_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string_pretty(snapshot) {
        let _ = fs::write(path, json);
    }
}

fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "kemoshumai", "pomoshumai")
        .map(|dirs| dirs.config_dir().join("settings.json"))
}

fn state_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "kemoshumai", "pomoshumai")
        .map(|dirs| dirs.config_dir().join("state.json"))
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
