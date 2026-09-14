use crate::Mode;
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Sense, Stroke, Vec2,
};
use std::{f32::consts::{FRAC_PI_2, TAU}, time::Duration};

pub const PAPER: Color32 = Color32::from_rgb(248, 246, 240);
pub const INK: Color32 = Color32::from_rgb(35, 39, 43);
pub const MUTED: Color32 = Color32::from_rgb(125, 127, 126);
pub const ACCENT: Color32 = Color32::from_rgb(160, 52, 47);

fn radial(center: Pos2, radius: f32, turns: f64) -> Pos2 {
    let angle = (turns.rem_euclid(1.0) as f32) * TAU - FRAC_PI_2;
    center + Vec2::new(angle.cos(), angle.sin()) * radius
}

fn draw_hand(
    painter: &egui::Painter,
    center: Pos2,
    length: f32,
    turns: f64,
    width: f32,
    color: Color32,
) {
    painter.line_segment(
        [
            radial(center, length * 0.16, turns + 0.5),
            radial(center, length, turns),
        ],
        Stroke::new(width, color),
    );
}

pub fn draw_dial(ui: &mut egui::Ui, size: f32, duration: Duration, mode: Mode) {
    let (response, painter) =
        ui.allocate_painter(Vec2::splat(size), Sense::hover());
    let center = response.rect.center();
    let radius = size * 0.405;
    let scale = size / 520.0;
    let seconds = duration.as_secs_f64();

    painter.circle_filled(
        center + Vec2::new(0.0, 5.0 * scale),
        radius * 1.075,
        Color32::from_black_alpha(16),
    );
    painter.circle_filled(center, radius * 1.07, INK);
    painter.circle_filled(center, radius * 1.025, PAPER);
    painter.circle_stroke(center, radius * 0.985, Stroke::new(1.0, MUTED));

    // Alternating railway-style bezel.
    for index in 0..120 {
        if index % 2 == 0 {
            painter.line_segment(
                [
                    radial(center, radius * 1.035, index as f64 / 120.0),
                    radial(center, radius * 1.06, index as f64 / 120.0),
                ],
                Stroke::new(5.0 * scale, PAPER),
            );
        }
    }

    // One subdivision every 0.2 seconds.
    for index in 0..300 {
        let major = index % 25 == 0;
        let second = index % 5 == 0;
        let inner = if major { 0.84 } else if second { 0.885 } else { 0.925 };
        painter.line_segment(
            [
                radial(center, radius * inner, index as f64 / 300.0),
                radial(center, radius * 0.97, index as f64 / 300.0),
            ],
            Stroke::new(
                if major { 2.0 * scale } else { 0.8 * scale },
                if second { INK } else { Color32::from_rgb(180, 177, 170) },
            ),
        );
    }

    for index in 0..12 {
        let value = if index == 0 { 60 } else { index * 5 };
        painter.text(
            radial(center, radius * 1.165, index as f64 / 12.0),
            Align2::CENTER_CENTER,
            value.to_string(),
            FontId::proportional(23.0 * scale),
            INK,
        );
    }

    // The small counter makes one full revolution in 30 minutes.
    let sub_center = center - Vec2::new(0.0, radius * 0.44);
    let sub_radius = radius * 0.245;
    painter.circle_filled(
        sub_center, sub_radius, Color32::from_rgb(238, 235, 226),
    );
    painter.circle_stroke(sub_center, sub_radius, Stroke::new(1.0, MUTED));

    for index in 0..30 {
        painter.line_segment(
            [
                radial(
                    sub_center,
                    sub_radius * if index % 5 == 0 { 0.77 } else { 0.88 },
                    index as f64 / 30.0,
                ),
                radial(sub_center, sub_radius, index as f64 / 30.0),
            ],
            Stroke::new(0.9 * scale, MUTED),
        );
    }

    for index in 0..6 {
        let value = if index == 0 { 30 } else { index * 5 };
        painter.text(
            radial(sub_center, sub_radius * 0.62, index as f64 / 6.0),
            Align2::CENTER_CENTER,
            value.to_string(),
            FontId::proportional(10.0 * scale),
            MUTED,
        );
    }

    draw_hand(
        &painter, sub_center, sub_radius * 0.77,
        (seconds % 1800.0) / 1800.0, 2.0 * scale, INK,
    );
    painter.circle_filled(sub_center, 3.0 * scale, INK);

    painter.text(
        center + Vec2::new(0.0, radius * 0.39),
        Align2::CENTER_CENTER,
        "CALIBRE 60",
        FontId::proportional(23.0 * scale),
        INK,
    );
    painter.text(
        center + Vec2::new(0.0, radius * 0.58),
        Align2::CENTER_CENTER,
        "JÉRÔMELAB",
        FontId::proportional(11.0 * scale),
        MUTED,
    );
    painter.text(
        center + Vec2::new(0.0, radius * 0.69),
        Align2::CENTER_CENTER,
        "60 SECONDES / 30 MINUTES",
        FontId::proportional(8.0 * scale),
        MUTED,
    );

    let hand_color = match mode {
        Mode::Stopwatch => INK,
        Mode::Countdown => ACCENT,
    };
    draw_hand(
        &painter, center, radius * 0.91,
        (seconds % 60.0) / 60.0, 3.5 * scale, hand_color,
    );
    painter.circle_filled(center, 8.0 * scale, hand_color);
    painter.circle_filled(center, 2.4 * scale, PAPER);
}
