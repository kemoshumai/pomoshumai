use std::f32::consts::TAU;
use std::time::Duration;

use gpui::PathBuilder;
use gpui::{Bounds, Hsla, Pixels, Window, canvas, point, px};

pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

pub fn progress_ratio(remaining: Duration, total: Duration) -> f32 {
    if total.is_zero() {
        return 0.0;
    }

    let total_secs = total.as_secs_f32();
    let remaining_secs = remaining.as_secs_f32().min(total_secs);
    let elapsed = total_secs - remaining_secs;
    (elapsed / total_secs).clamp(0.0, 1.0)
}

pub fn timer_ring(
    progress: f32,
    stroke_width: f32,
    base_color: Hsla,
    progress_color: Hsla,
) -> gpui::Canvas<f32> {
    let progress = progress.clamp(0.0, 1.0);

    canvas(
        move |_, _, _| progress,
        move |bounds, progress, window, _| {
            draw_ring(
                bounds,
                stroke_width,
                base_color,
                progress_color,
                progress,
                window,
            );
        },
    )
}

fn draw_ring(
    bounds: Bounds<Pixels>,
    stroke_width: f32,
    base_color: Hsla,
    progress_color: Hsla,
    progress: f32,
    window: &mut Window,
) {
    let center = bounds.center();
    let center_x = f32::from(center.x);
    let center_y = f32::from(center.y);
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let radius = (width.min(height) - stroke_width) / 2.0;

    if radius <= 0.0 {
        return;
    }

    let start_angle = -std::f32::consts::FRAC_PI_2;

    if let Some(path) = arc_path(
        center_x,
        center_y,
        radius,
        start_angle,
        start_angle + TAU,
        stroke_width,
    ) {
        window.paint_path(path, base_color);
    }

    if progress > 0.0 {
        let end_angle = start_angle + TAU * progress;
        if let Some(path) = arc_path(
            center_x,
            center_y,
            radius,
            start_angle,
            end_angle,
            stroke_width,
        ) {
            window.paint_path(path, progress_color);
        }
    }
}

fn arc_path(
    center_x: f32,
    center_y: f32,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    stroke_width: f32,
) -> Option<gpui::Path<Pixels>> {
    let delta = (end_angle - start_angle).clamp(0.0, TAU);
    if delta <= f32::EPSILON {
        return None;
    }

    let radii = point(px(radius), px(radius));
    let start = point(
        px(center_x + radius * start_angle.cos()),
        px(center_y + radius * start_angle.sin()),
    );

    let mut builder = PathBuilder::stroke(px(stroke_width));

    if (TAU - delta).abs() < 0.001 {
        let mid_angle = start_angle + std::f32::consts::PI;
        let mid = point(
            px(center_x + radius * mid_angle.cos()),
            px(center_y + radius * mid_angle.sin()),
        );

        builder.move_to(start);
        builder.arc_to(radii, px(0.), false, true, mid);
        builder.arc_to(radii, px(0.), false, true, start);
    } else {
        let end = point(
            px(center_x + radius * end_angle.cos()),
            px(center_y + radius * end_angle.sin()),
        );
        let large_arc = delta > std::f32::consts::PI;

        builder.move_to(start);
        builder.arc_to(radii, px(0.), large_arc, true, end);
    }

    builder.build().ok()
}
