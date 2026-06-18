use gpui::prelude::FluentBuilder as _;
use gpui::{
    Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb, rgba,
};
use gpui_component::button::Button;

use super::PomodoroApp;
use super::state::{Phase, TimerStatus};
use super::ui::{format_duration, progress_ratio, timer_ring};

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
        let _current_pomodoro_index = self.current_pomodoro_index();

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
