use gpui::{ClickEvent, Context, Window};

use super::PomodoroApp;

impl PomodoroApp {
    pub(super) fn on_play_clicked(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let now = cx.background_executor().now();
        self.timer.start(now);
        self.save_snapshot();
        cx.notify();
    }

    pub(super) fn on_pause_clicked(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let now = cx.background_executor().now();
        self.timer.pause(now);
        self.save_snapshot();
        cx.notify();
    }

    pub(super) fn on_cancel_clicked(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.timer.cancel(&self.settings);
        self.save_snapshot();
        cx.notify();
    }

    pub(super) fn on_open_settings(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = true;
        cx.notify();
    }

    pub(super) fn on_close_settings(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = false;
        cx.notify();
    }

    pub(super) fn on_reset_settings(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reset_settings(window, cx);
    }

    pub(super) fn on_toggle_discord(
        &mut self,
        checked: &bool,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_discord_presence(*checked, cx);
    }

    pub(super) fn on_tweet_clicked(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = format!("今日の成果: {} ポモドーロ", self.completed_pomodoros);
        let url = format!(
            "https://twitter.com/intent/tweet?text={}",
            urlencoding::encode(&text)
        );
        cx.open_url(&url);
    }

    pub(super) fn on_reset_results_clicked(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.confirm_reset_results = true;
        cx.notify();
    }

    pub(super) fn on_cancel_reset_results(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.confirm_reset_results = false;
        cx.notify();
    }

    pub(super) fn on_confirm_reset_results(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.completed_pomodoros = 0;
        self.timer.cancel(&self.settings);
        self.confirm_reset_results = false;
        self.save_snapshot();
        self.update_discord_presence();
        cx.notify();
    }
}
