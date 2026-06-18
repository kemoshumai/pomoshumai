mod actions;
mod discord;
mod lifecycle;
mod render;
mod settings;
mod state;
mod storage;
mod timer;
mod ui;

use gpui::{Entity, Subscription, Task};
use gpui_component::input::InputState;

use discord::DiscordPresence;
use state::Settings;
use timer::TimerState;

pub struct PomodoroApp {
    pub(super) settings: Settings,
    pub(super) timer: TimerState,
    pub(super) completed_pomodoros: u32,
    pub(super) show_settings: bool,
    pub(super) confirm_reset_results: bool,
    pub(super) settings_inputs: Option<SettingsInputs>,
    pub(super) tick_task: Option<Task<()>>,
    pub(super) presence_task: Option<Task<()>>,
    pub(super) discord: DiscordPresence,
}

pub(super) struct SettingsInputs {
    pub(super) work: Entity<InputState>,
    pub(super) short_break: Entity<InputState>,
    pub(super) long_break: Entity<InputState>,
    pub(super) long_break_frequency: Entity<InputState>,
    pub(super) _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
pub(super) enum SettingField {
    Work,
    ShortBreak,
    LongBreak,
    LongBreakFrequency,
}
