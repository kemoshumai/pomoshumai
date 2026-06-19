#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;

use gpui::{
    App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(480.), px(640.)), cx);
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("pomoshumai".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(app::PomodoroApp::new);
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
    })
}
