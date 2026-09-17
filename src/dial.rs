use crate::{app::Mode, settings::DialStyle};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Stroke, Vec2};
use std::{
    f32::consts::{FRAC_PI_2, TAU},
    time::Duration,
};

pub const PAPER: Color32 = Color32::from_rgb(248, 246, 240);
pub const INK: Color32 = Color32::from_rgb(35, 39, 43);
pub const MUTED: Color32 = Color32::from_rgb(125, 127, 126);
pub const ACCENT: Color32 = Color32::from_rgb(160, 52, 47);

const NAVY: Color32 = Color32::from_rgb(18, 44, 78);
const NAVY_SUBDIAL: Color32 = Color32::from_rgb(22, 53, 91);
const NAVY_BEZEL: Color32 = Color32::from_rgb(14, 22, 34);
const NAVY_MUTED: Color32 = Color32::from_rgb(170, 190, 211);
const NAVY_FINE_TICK: Color32 = Color32::from_rgb(106, 139, 172);
const RACING_YELLOW: Color32 = Color32::from_rgb(232, 190, 48);
const RACING_RED: Color32 = Color32::from_rgb(210, 52, 43);

#[derive(Clone, Copy)]
struct DialPalette {
    face: Color32,
    bezel: Color32,
    text: Color32,
    muted: Color32,
    fine_tick: Color32,
    sub_face: Color32,
    sub_text: Color32,
    bezel_tick: Color32,
    stopwatch_hand: Color32,
    countdown_hand: Color32,
}

impl DialPalette {
    fn for_style(style: DialStyle) -> Self {
        match style {
            DialStyle::Classic => Self {
                face: PAPER,
                bezel: INK,
                text: INK,
                muted: MUTED,
                fine_tick: Color32::from_rgb(180, 177, 170),
                sub_face: Color32::from_rgb(238, 235, 226),
                sub_text: MUTED,
                bezel_tick: PAPER,
                stopwatch_hand: INK,
                countdown_hand: ACCENT,
            },
            DialStyle::Navy => Self {
                face: NAVY,
                bezel: NAVY_BEZEL,
                text: PAPER,
                muted: NAVY_MUTED,
                fine_tick: NAVY_FINE_TICK,
                sub_face: NAVY_SUBDIAL,
                sub_text: PAPER,
                bezel_tick: PAPER,
                stopwatch_hand: PAPER,
                countdown_hand: RACING_RED,
            },
        }
    }
}

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

fn draw_navy_subdial_bezel(painter: &egui::Painter, center: Pos2, radius: f32, scale: f32) {
    painter.circle_stroke(center, radius * 1.16, Stroke::new(1.2 * scale, NAVY_BEZEL));

    // Colour is deliberately limited to the small 30-minute counter:
    // three quarters yellow, followed by one quarter red.
    const TOTAL_TICKS: usize = 60;
    const YELLOW_TICKS: usize = TOTAL_TICKS * 3 / 4;

    for index in 0..TOTAL_TICKS {
        let color = if index < YELLOW_TICKS {
            RACING_YELLOW
        } else {
            RACING_RED
        };
        painter.line_segment(
            [
                radial(center, radius * 1.025, index as f64 / TOTAL_TICKS as f64),
                radial(center, radius * 1.14, index as f64 / TOTAL_TICKS as f64),
            ],
            Stroke::new(3.2 * scale, color),
        );
    }
}

pub fn draw_dial(ui: &mut egui::Ui, size: f32, duration: Duration, mode: Mode, style: DialStyle) {
    let (response, painter) = ui.allocate_painter(Vec2::splat(size), Sense::hover());
    let center = response.rect.center();
    let radius = size * 0.405;
    let scale = size / 520.0;
    let seconds = duration.as_secs_f64();
    let palette = DialPalette::for_style(style);

    painter.circle_filled(
        center + Vec2::new(0.0, 5.0 * scale),
        radius * 1.075,
        Color32::from_black_alpha(if style == DialStyle::Navy { 38 } else { 16 }),
    );
    painter.circle_filled(center, radius * 1.07, palette.bezel);
    if style == DialStyle::Navy {
        painter.circle_stroke(
            center,
            radius * 1.072,
            Stroke::new(4.0 * scale, RACING_RED),
        );
    }
    painter.circle_filled(center, radius * 1.025, palette.face);
    painter.circle_stroke(center, radius * 0.985, Stroke::new(1.0_f32, palette.muted));

    // The main outer bezel keeps the original neutral railway treatment.
    for index in 0..120 {
        if index % 2 == 0 {
            painter.line_segment(
                [
                    radial(center, radius * 1.035, index as f64 / 120.0),
                    radial(center, radius * 1.06, index as f64 / 120.0),
                ],
                Stroke::new(5.0 * scale, palette.bezel_tick),
            );
        }
    }

    // One subdivision every 0.2 seconds.
    for index in 0..300 {
        let major = index % 25 == 0;
        let second = index % 5 == 0;
        let inner = if major {
            0.84
        } else if second {
            0.885
        } else {
            0.925
        };
        painter.line_segment(
            [
                radial(center, radius * inner, index as f64 / 300.0),
                radial(center, radius * 0.97, index as f64 / 300.0),
            ],
            Stroke::new(
                if major { 2.0 * scale } else { 0.8 * scale },
                if second {
                    palette.text
                } else {
                    palette.fine_tick
                },
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
            palette.text,
        );
    }

    // The small counter makes one full revolution in 30 minutes.
    let sub_center = center - Vec2::new(0.0, radius * 0.44);
    let sub_radius = radius * 0.245;
    painter.circle_filled(sub_center, sub_radius, palette.sub_face);

    if style == DialStyle::Navy {
        draw_navy_subdial_bezel(&painter, sub_center, sub_radius, scale);
    } else {
        painter.circle_stroke(sub_center, sub_radius, Stroke::new(1.0_f32, palette.muted));
    }

    for index in 0..30 {
        painter.line_segment(
            [
                radial(
                    sub_center,
                    sub_radius * if index % 5 == 0 { 0.77 } else { 0.88 },
                    index as f64 / 30.0,
                ),
                radial(sub_center, sub_radius * 0.98, index as f64 / 30.0),
            ],
            Stroke::new(0.9 * scale, palette.muted),
        );
    }

    for index in 0..6 {
        let value = if index == 0 { 30 } else { index * 5 };
        painter.text(
            radial(sub_center, sub_radius * 0.62, index as f64 / 6.0),
            Align2::CENTER_CENTER,
            value.to_string(),
            FontId::proportional(10.0 * scale),
            palette.sub_text,
        );
    }

    draw_hand(
        &painter,
        sub_center,
        sub_radius * 0.77,
        (seconds % 1800.0) / 1800.0,
        2.0 * scale,
        palette.text,
    );
    painter.circle_filled(sub_center, 3.0 * scale, palette.text);

    painter.text(
        center + Vec2::new(0.0, radius * 0.39),
        Align2::CENTER_CENTER,
        "CALIBRE 60",
        FontId::proportional(23.0 * scale),
        palette.text,
    );
    painter.text(
        center + Vec2::new(0.0, radius * 0.58),
        Align2::CENTER_CENTER,
        "JÉRÔMELAB",
        FontId::proportional(11.0 * scale),
        palette.muted,
    );
    painter.text(
        center + Vec2::new(0.0, radius * 0.69),
        Align2::CENTER_CENTER,
        "60 SECONDES / 30 MINUTES",
        FontId::proportional(8.0 * scale),
        palette.muted,
    );

    let hand_color = match mode {
        Mode::Stopwatch => palette.stopwatch_hand,
        Mode::Countdown => palette.countdown_hand,
    };
    draw_hand(
        &painter,
        center,
        radius * 0.91,
        (seconds % 60.0) / 60.0,
        3.5 * scale,
        hand_color,
    );
    painter.circle_filled(center, 8.0 * scale, hand_color);
    painter.circle_filled(center, 2.4 * scale, palette.face);
}
