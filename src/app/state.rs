use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub work_minutes: u32,
    pub break_minutes: u32,
    pub long_break_minutes: u32,
    pub long_break_frequency: u32,
    pub discord_presence: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            break_minutes: 5,
            long_break_minutes: 15,
            long_break_frequency: 4,
            discord_presence: false,
        }
    }
}

impl Settings {
    pub fn duration_for_phase(&self, phase: Phase) -> Duration {
        let minutes = match phase {
            Phase::Work => self.work_minutes,
            Phase::ShortBreak => self.break_minutes,
            Phase::LongBreak => self.long_break_minutes,
        };
        Duration::from_secs(minutes as u64 * 60)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
}
