use std::time::Duration;

use gpui::{AsyncApp, Context, WeakEntity};

use super::state::{Phase, TimerStatus};
use super::storage;
use super::timer::TimerState;
use super::{DiscordPresence, PomodoroApp};

impl PomodoroApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let settings = storage::load_settings();
        let mut app = Self {
            timer: TimerState::new(&settings),
            settings,
            completed_pomodoros: 0,
            show_settings: false,
            settings_inputs: None,
            tick_task: None,
            presence_task: None,
            discord: DiscordPresence::new(),
        };

        app.spawn_tick(cx);
        app.spawn_presence(cx);
        app.update_discord_presence();
        app
    }

    fn spawn_tick(&mut self, cx: &mut Context<Self>) {
        if self.tick_task.is_some() {
            return;
        }

        let task = cx.spawn(|this: WeakEntity<PomodoroApp>, cx: &mut AsyncApp| {
            let mut app = cx.clone();
            async move {
                loop {
                    app.background_executor()
                        .timer(Duration::from_secs(1))
                        .await;
                    let Some(this) = this.upgrade() else {
                        break;
                    };
                    let _ = this.update(&mut app, |state, cx| state.handle_tick(cx));
                }
            }
        });

        self.tick_task = Some(task);
    }

    fn spawn_presence(&mut self, cx: &mut Context<Self>) {
        if self.presence_task.is_some() {
            return;
        }

        let task = cx.spawn(|this: WeakEntity<PomodoroApp>, cx: &mut AsyncApp| {
            let mut app = cx.clone();
            async move {
                loop {
                    let Some(this) = this.upgrade() else {
                        break;
                    };
                    let _ = this.update(&mut app, |state, _| state.update_discord_presence());
                    app.background_executor()
                        .timer(Duration::from_secs(15))
                        .await;
                }
            }
        });

        self.presence_task = Some(task);
    }

    fn handle_tick(&mut self, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        let finished = self.timer.tick(now);

        if finished {
            match self.timer.phase {
                Phase::Work => {
                    self.completed_pomodoros += 1;
                    let frequency = self.settings.long_break_frequency.max(1);
                    let next_phase = if self.completed_pomodoros.is_multiple_of(frequency) {
                        Phase::LongBreak
                    } else {
                        Phase::ShortBreak
                    };
                    self.timer.set_phase(next_phase, &self.settings);
                }
                Phase::ShortBreak | Phase::LongBreak => {
                    self.timer.set_phase(Phase::Work, &self.settings);
                }
            }
        }

        if self.timer.status == TimerStatus::Running || finished {
            cx.notify();
        }
    }

    pub(super) fn update_discord_presence(&mut self) {
        let status = self.discord_status_text();
        self.discord.update(
            &self.settings,
            &self.timer,
            status,
            self.completed_pomodoros,
        );
    }

    pub(super) fn status_text(&self) -> &'static str {
        match self.timer.status {
            TimerStatus::Paused => "一時停止",
            TimerStatus::Running => {
                if self.timer.phase == Phase::Work {
                    "作業中"
                } else {
                    "休憩中"
                }
            }
            TimerStatus::Idle => {
                if self.timer.phase == Phase::Work {
                    "作業開始"
                } else {
                    "休憩開始"
                }
            }
        }
    }

    fn discord_status_text(&self) -> &'static str {
        match self.timer.status {
            TimerStatus::Paused => "一時停止中",
            TimerStatus::Running => {
                if self.timer.phase == Phase::Work {
                    "作業中"
                } else {
                    "休憩中"
                }
            }
            TimerStatus::Idle => {
                if self.timer.phase == Phase::Work {
                    "作業前"
                } else {
                    "休憩前"
                }
            }
        }
    }

    pub(super) fn current_pomodoro_index(&self) -> u32 {
        if self.timer.phase == Phase::Work {
            self.completed_pomodoros + 1
        } else {
            self.completed_pomodoros.max(1)
        }
    }

    pub(super) fn update_idle_timer_duration(&mut self) {
        if self.timer.status == TimerStatus::Idle {
            let duration = self.settings.duration_for_phase(self.timer.phase);
            self.timer.session_duration = duration;
            self.timer.remaining = duration;
        }
    }
}
