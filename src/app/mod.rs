mod discord;
mod state;
mod storage;
mod timer;
mod ui;

use std::time::Duration;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, AsyncApp, ClickEvent, Context, Entity, FontWeight, IntoElement, ParentElement,
    Render, Styled, Subscription, Task, WeakEntity, Window, div, px, rgb, rgba,
};
use gpui_component::button::Button;
use gpui_component::input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction};
use gpui_component::scroll::ScrollableElement;
use gpui_component::switch::Switch;
use gpui_component::{Sizable, Size};

use discord::DiscordPresence;
use state::{Phase, Settings, TimerStatus};
use timer::TimerState;
use ui::{format_duration, progress_ratio, timer_ring};

pub struct PomodoroApp {
    settings: Settings,
    timer: TimerState,
    completed_pomodoros: u32,
    show_settings: bool,
    settings_inputs: Option<SettingsInputs>,
    tick_task: Option<Task<()>>,
    presence_task: Option<Task<()>>,
    discord: DiscordPresence,
}

struct SettingsInputs {
    work: Entity<InputState>,
    short_break: Entity<InputState>,
    long_break: Entity<InputState>,
    long_break_frequency: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
enum SettingField {
    Work,
    ShortBreak,
    LongBreak,
    LongBreakFrequency,
}

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

    fn update_discord_presence(&mut self) {
        let status = self.status_text();
        self.discord.update(
            &self.settings,
            &self.timer,
            status,
            self.completed_pomodoros,
        );
    }

    fn status_text(&self) -> &'static str {
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

    fn current_pomodoro_index(&self) -> u32 {
        if self.timer.phase == Phase::Work {
            self.completed_pomodoros + 1
        } else {
            self.completed_pomodoros.max(1)
        }
    }

    fn ensure_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.settings_inputs.is_none() {
            let work = cx.new(|cx| {
                InputState::new(window, cx).default_value(self.settings.work_minutes.to_string())
            });
            let short_break = cx.new(|cx| {
                InputState::new(window, cx).default_value(self.settings.break_minutes.to_string())
            });
            let long_break = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(self.settings.long_break_minutes.to_string())
            });
            let long_break_frequency = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(self.settings.long_break_frequency.to_string())
            });

            let mut subscriptions = Vec::new();
            subscriptions.extend(self.bind_number_input(
                &work,
                window,
                cx,
                SettingField::Work,
                1,
                180,
            ));
            subscriptions.extend(self.bind_number_input(
                &short_break,
                window,
                cx,
                SettingField::ShortBreak,
                1,
                60,
            ));
            subscriptions.extend(self.bind_number_input(
                &long_break,
                window,
                cx,
                SettingField::LongBreak,
                1,
                90,
            ));
            subscriptions.extend(self.bind_number_input(
                &long_break_frequency,
                window,
                cx,
                SettingField::LongBreakFrequency,
                1,
                12,
            ));

            self.settings_inputs = Some(SettingsInputs {
                work,
                short_break,
                long_break,
                long_break_frequency,
                _subscriptions: subscriptions,
            });
        }

        self.sync_settings_inputs(window, cx);
    }

    fn bind_number_input(
        &mut self,
        input: &Entity<InputState>,
        window: &mut Window,
        cx: &mut Context<Self>,
        field: SettingField,
        min: u32,
        max: u32,
    ) -> Vec<Subscription> {
        let min_i64 = min as i64;
        let max_i64 = max as i64;

        let mut subscriptions = Vec::new();

        let step_sub = cx.subscribe_in(input, window, move |_, input, event, window, cx| {
            let NumberInputEvent::Step(action) = event;
            input.update(cx, |input, cx| {
                let current = input
                    .value()
                    .parse::<i64>()
                    .unwrap_or(min_i64)
                    .clamp(min_i64, max_i64);
                let delta = if *action == StepAction::Increment {
                    1
                } else {
                    -1
                };
                let next = (current + delta).clamp(min_i64, max_i64);
                input.set_value(next.to_string(), window, cx);
            });
        });

        let change_sub = cx.subscribe_in(input, window, move |this, input, event, window, cx| {
            if !matches!(event, InputEvent::Change) {
                return;
            }

            let mut parsed = None;
            input.update(cx, |input, cx| {
                let raw = input.value();
                if let Ok(value) = raw.parse::<u32>() {
                    let clamped = value.clamp(min, max);
                    if clamped != value {
                        input.set_value(clamped.to_string(), window, cx);
                    }
                    parsed = Some(clamped);
                }
            });

            if let Some(value) = parsed {
                this.update_setting_field(field, value, cx);
            }
        });

        subscriptions.push(step_sub);
        subscriptions.push(change_sub);
        subscriptions
    }

    fn update_setting_field(&mut self, field: SettingField, value: u32, cx: &mut Context<Self>) {
        let mut changed = false;

        match field {
            SettingField::Work => {
                if self.settings.work_minutes != value {
                    self.settings.work_minutes = value;
                    changed = true;
                }
            }
            SettingField::ShortBreak => {
                if self.settings.break_minutes != value {
                    self.settings.break_minutes = value;
                    changed = true;
                }
            }
            SettingField::LongBreak => {
                if self.settings.long_break_minutes != value {
                    self.settings.long_break_minutes = value;
                    changed = true;
                }
            }
            SettingField::LongBreakFrequency => {
                let value = value.max(1);
                if self.settings.long_break_frequency != value {
                    self.settings.long_break_frequency = value;
                    changed = true;
                }
            }
        }

        if changed {
            storage::save_settings(&self.settings);
            self.update_idle_timer_duration();
            self.update_discord_presence();
            cx.notify();
        }
    }

    fn update_idle_timer_duration(&mut self) {
        if self.timer.status == TimerStatus::Idle {
            let duration = self.settings.duration_for_phase(self.timer.phase);
            self.timer.session_duration = duration;
            self.timer.remaining = duration;
        }
    }

    fn sync_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inputs) = &self.settings_inputs else {
            return;
        };

        sync_input_value(&inputs.work, self.settings.work_minutes, window, cx);
        sync_input_value(&inputs.short_break, self.settings.break_minutes, window, cx);
        sync_input_value(
            &inputs.long_break,
            self.settings.long_break_minutes,
            window,
            cx,
        );
        sync_input_value(
            &inputs.long_break_frequency,
            self.settings.long_break_frequency,
            window,
            cx,
        );
    }

    fn on_play_clicked(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        self.timer.start(now);
        cx.notify();
    }

    fn on_pause_clicked(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        self.timer.pause(now);
        cx.notify();
    }

    fn on_cancel_clicked(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.timer.cancel(&self.settings);
        cx.notify();
    }

    fn on_open_settings(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.show_settings = true;
        cx.notify();
    }

    fn on_close_settings(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.show_settings = false;
        cx.notify();
    }

    fn on_reset_settings(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = Settings::default();
        storage::save_settings(&self.settings);
        self.update_idle_timer_duration();
        self.sync_settings_inputs(window, cx);
        self.update_discord_presence();
        cx.notify();
    }

    fn on_toggle_discord(&mut self, checked: &bool, _: &mut Window, cx: &mut Context<Self>) {
        if self.settings.discord_presence != *checked {
            self.settings.discord_presence = *checked;
            storage::save_settings(&self.settings);
        }
        self.update_discord_presence();
        cx.notify();
    }

    fn on_tweet_clicked(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let text = format!("今日の成果: {} ポモドーロ", self.completed_pomodoros);
        let url = format!(
            "https://twitter.com/intent/tweet?text={}",
            urlencoding::encode(&text)
        );
        cx.open_url(&url);
    }

    fn render_settings_overlay(
        &mut self,
        settings_width: f32,
        settings_input_width: f32,
        fullscreen: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(inputs) = &self.settings_inputs else {
            return div();
        };

        let input_size = Size::Small;

        div()
            .absolute()
            .inset_0()
            .bg(if fullscreen {
                rgb(0xFFFFFF).into()
            } else {
                rgba(0x00000066)
            })
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .bg(rgb(0xFFFFFF))
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .overflow_y_scrollbar()
                    .when(fullscreen, |panel| panel.w_full().h_full())
                    .when(!fullscreen, |panel| {
                        panel.rounded_lg().w(px(settings_width)).h_auto()
                    })
                    .child(
                        div()
                            .text_size(px(20.))
                            .font_weight(FontWeight::BOLD)
                            .child("設定"),
                    )
                    .child(setting_row(
                        "作業",
                        NumberInput::new(&inputs.work)
                            .with_size(input_size)
                            .w(px(settings_input_width))
                            .suffix(div().px_2().child("分")),
                    ))
                    .child(setting_row(
                        "休憩",
                        NumberInput::new(&inputs.short_break)
                            .with_size(input_size)
                            .w(px(settings_input_width))
                            .suffix(div().px_2().child("分")),
                    ))
                    .child(setting_row(
                        "長時間休憩",
                        NumberInput::new(&inputs.long_break)
                            .with_size(input_size)
                            .w(px(settings_input_width))
                            .suffix(div().px_2().child("分")),
                    ))
                    .child(setting_row(
                        "長時間休憩の頻度",
                        NumberInput::new(&inputs.long_break_frequency)
                            .with_size(input_size)
                            .w(px(settings_input_width))
                            .suffix(div().px_2().child("ポモドーロ")),
                    ))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(div().text_sm().child("Discordのステータスに表示"))
                            .child(
                                Switch::new("discord-toggle")
                                    .checked(self.settings.discord_presence)
                                    .on_click(cx.listener(Self::on_toggle_discord)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .child(
                                Button::new("settings-reset")
                                    .label("デフォルトに戻す")
                                    .on_click(cx.listener(Self::on_reset_settings)),
                            )
                            .child(
                                Button::new("settings-close")
                                    .label("閉じる")
                                    .on_click(cx.listener(Self::on_close_settings)),
                            ),
                    ),
            )
    }
}

impl Render for PomodoroApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.show_settings {
            self.ensure_settings_inputs(window, cx);
        }

        let viewport_size = window.viewport_size();
        let viewport_width = f32::from(viewport_size.width);
        let viewport_height = f32::from(viewport_size.height);
        let edge_padding = 10.0;
        let content_gap = 16.0;
        let controls_height = 112.0;
        let available_width = (viewport_width - edge_padding * 2.0).max(0.0);
        let available_height =
            (viewport_height - edge_padding * 2.0 - controls_height - content_gap * 2.0).max(0.0);
        let ring_size = available_width.min(available_height).clamp(120.0, 300.0);
        let ring_stroke_width = (ring_size * 0.04).clamp(6.0, 12.0);
        let time_text_size = (ring_size * 0.16).clamp(24.0, 48.0);
        let settings_fullscreen = viewport_width < 420.0 || viewport_height < 560.0;
        let settings_width = if settings_fullscreen {
            available_width
        } else {
            available_width.clamp(260.0, 380.0)
        };
        let settings_input_width = (settings_width * 0.42).clamp(120.0, 160.0);

        let background = if self.timer.phase == Phase::Work {
            rgb(0xFFEFF2)
        } else {
            rgb(0xE8F7EF)
        };

        let ring_base = rgba(0xFFFFFF55);
        let ring_progress = rgb(0xFFFFFF);

        let progress = progress_ratio(self.timer.remaining, self.timer.session_duration);
        let time_text = format_duration(self.timer.remaining);
        let status_text = self.status_text();
        let completed = self.completed_pomodoros;

        let controls = match self.timer.status {
            TimerStatus::Running => div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    Button::new("pause")
                        .label("停止")
                        .on_click(cx.listener(Self::on_pause_clicked)),
                )
                .child(
                    Button::new("cancel")
                        .label("中止")
                        .on_click(cx.listener(Self::on_cancel_clicked)),
                ),
            TimerStatus::Paused => div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    Button::new("resume")
                        .label("再開")
                        .on_click(cx.listener(Self::on_play_clicked)),
                )
                .child(
                    Button::new("cancel")
                        .label("中止")
                        .on_click(cx.listener(Self::on_cancel_clicked)),
                ),
            TimerStatus::Idle => div().flex().items_center().gap_4().child(
                Button::new("play")
                    .label("再生")
                    .on_click(cx.listener(Self::on_play_clicked)),
            ),
        };

        div()
            .relative()
            .w_full()
            .h_full()
            .bg(background)
            .text_color(rgb(0x1A1A1A))
            .p(px(edge_padding))
            .child(
                Button::new("settings-button")
                    .label("設定")
                    .on_click(cx.listener(Self::on_open_settings))
                    .absolute()
                    .top(px(edge_padding))
                    .right(px(edge_padding)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .h_full()
                    .child(
                        div()
                            .relative()
                            .w(px(ring_size))
                            .h(px(ring_size))
                            .child(
                                timer_ring(
                                    progress,
                                    ring_stroke_width,
                                    ring_base.into(),
                                    ring_progress.into(),
                                )
                                    .absolute()
                                    .inset_0(),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .inset_0()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_size(px(time_text_size))
                                            .font_weight(FontWeight::BOLD)
                                            .child(time_text),
                                    )
                                    .child(div().text_sm().child(status_text)),
                            ),
                    )
                    .child(controls)
                    .child(
                        div()
                            .text_sm()
                            .child(format!("現在 {completed} ポモドーロ作業済み")),
                    )
                    .child(
                        Button::new("tweet")
                            .label("今日の成果をツイートする")
                            .on_click(cx.listener(Self::on_tweet_clicked)),
                    ),
            )
            .when(self.show_settings, |this| {
                this.child(self.render_settings_overlay(
                    settings_width,
                    settings_input_width,
                    settings_fullscreen,
                    window,
                    cx,
                ))
            })
    }
}

fn setting_row(label: impl Into<String>, input: impl IntoElement) -> impl IntoElement {
    let label = label.into();
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(div().text_sm().child(label))
        .child(input)
}

fn sync_input_value(input: &Entity<InputState>, value: u32, window: &mut Window, cx: &mut App) {
    let value = value.to_string();
    input.update(cx, |input, cx| {
        if input.value().as_ref() == value.as_str() {
            return;
        }
        input.set_value(value.clone(), window, cx);
    });
}
