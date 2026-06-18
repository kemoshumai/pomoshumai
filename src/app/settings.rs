use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Styled, Subscription,
    Window, div, px, rgb, rgba,
};
use gpui_component::button::Button;
use gpui_component::input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction};
use gpui_component::scroll::ScrollableElement;
use gpui_component::switch::Switch;
use gpui_component::{Sizable, Size};

use super::state::Settings;
use super::storage;
use super::{PomodoroApp, SettingField, SettingsInputs};

impl PomodoroApp {
    pub(super) fn ensure_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub(super) fn sync_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub(super) fn reset_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = Settings::default();
        storage::save_settings(&self.settings);
        self.update_idle_timer_duration();
        self.sync_settings_inputs(window, cx);
        self.update_discord_presence();
        cx.notify();
    }

    pub(super) fn set_discord_presence(&mut self, checked: bool, cx: &mut Context<Self>) {
        if self.settings.discord_presence != checked {
            self.settings.discord_presence = checked;
            storage::save_settings(&self.settings);
        }
        self.update_discord_presence();
        cx.notify();
    }

    pub(super) fn render_settings_overlay(
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
                rgb(0xFFFFFF)
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
