from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"Expected one anchor in {path}, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


settings = r'''use crate::i18n::Language;
use serde::{Deserialize, Serialize};

pub const MAX_COUNTDOWN_SECONDS: u64 = 99 * 3_600 + 59 * 60 + 59;
pub const MIN_ALARM_REPEAT_MS: u64 = 500;
pub const MAX_ALARM_REPEAT_MS: u64 = 5_000;
pub const DEFAULT_PRESETS_MINUTES: [u64; 5] = [1, 3, 5, 10, 25];
const MAX_PRESET_MINUTES: u64 = MAX_COUNTDOWN_SECONDS / 60;
const STORAGE_KEY: &str = "calibre60.preferences";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedMode {
    #[default]
    Stopwatch,
    Countdown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiTheme {
    #[default]
    Light,
    Dark,
}

impl UiTheme {
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    pub fn label(self, language: Language) -> &'static str {
        match self {
            Self::Light => language.tr("light_mode"),
            Self::Dark => language.tr("dark_mode"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialStyle {
    #[default]
    Classic,
    Navy,
}

impl DialStyle {
    pub const ALL: [Self; 2] = [Self::Classic, Self::Navy];

    pub fn label(self, language: Language) -> &'static str {
        match self {
            Self::Classic => language.tr("dial_classic"),
            Self::Navy => language.tr("dial_navy"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmTone {
    #[default]
    Classic,
    DoubleBeep,
    Chime,
    Digital,
}

impl AlarmTone {
    pub const ALL: [Self; 4] = [Self::Classic, Self::DoubleBeep, Self::Chime, Self::Digital];

    pub fn label(self, language: Language) -> &'static str {
        match self {
            Self::Classic => language.tr("classic"),
            Self::DoubleBeep => language.tr("double_beep"),
            Self::Chime => language.tr("chime"),
            Self::Digital => language.tr("digital"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub mode: SavedMode,
    pub sound: bool,
    pub countdown_seconds: u64,
    pub alarm_tone: AlarmTone,
    pub alarm_volume: u8,
    pub alarm_repeat_ms: u64,
    pub language: Language,
    pub presets_minutes: [u64; 5],
    pub ui_theme: UiTheme,
    pub dial_style: DialStyle,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            mode: SavedMode::Stopwatch,
            sound: true,
            countdown_seconds: 5 * 60,
            alarm_tone: AlarmTone::Classic,
            alarm_volume: 35,
            alarm_repeat_ms: 1_000,
            language: Language::French,
            presets_minutes: DEFAULT_PRESETS_MINUTES,
            ui_theme: UiTheme::Light,
            dial_style: DialStyle::Classic,
        }
    }
}

impl Preferences {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|storage| eframe::get_value::<Self>(storage, STORAGE_KEY))
            .unwrap_or_default()
            .sanitized()
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.clone().sanitized());
    }

    fn sanitized(mut self) -> Self {
        self.countdown_seconds = self.countdown_seconds.min(MAX_COUNTDOWN_SECONDS);
        self.alarm_volume = self.alarm_volume.min(100);
        self.alarm_repeat_ms = self
            .alarm_repeat_ms
            .clamp(MIN_ALARM_REPEAT_MS, MAX_ALARM_REPEAT_MS);
        for preset in &mut self.presets_minutes {
            *preset = (*preset).clamp(1, MAX_PRESET_MINUTES);
        }
        self
    }
}

pub fn split_duration(total_seconds: u64) -> [u64; 3] {
    let total_seconds = total_seconds.min(MAX_COUNTDOWN_SECONDS);
    [
        total_seconds / 3_600,
        (total_seconds / 60) % 60,
        total_seconds % 60,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences_match_the_original_application_defaults() {
        let preferences = Preferences::default();
        assert_eq!(preferences.mode, SavedMode::Stopwatch);
        assert!(preferences.sound);
        assert_eq!(preferences.countdown_seconds, 300);
        assert_eq!(preferences.alarm_tone, AlarmTone::Classic);
        assert_eq!(preferences.alarm_volume, 35);
        assert_eq!(preferences.alarm_repeat_ms, 1_000);
        assert_eq!(preferences.language, Language::French);
        assert_eq!(preferences.presets_minutes, [1, 3, 5, 10, 25]);
        assert_eq!(preferences.ui_theme, UiTheme::Light);
        assert_eq!(preferences.dial_style, DialStyle::Classic);
    }

    #[test]
    fn split_duration_clamps_to_the_supported_maximum() {
        assert_eq!(split_duration(3_661), [1, 1, 1]);
        assert_eq!(split_duration(u64::MAX), [99, 59, 59]);
    }

    #[test]
    fn alarm_preferences_are_clamped_to_safe_ui_limits() {
        let preferences = Preferences {
            alarm_volume: 250,
            alarm_repeat_ms: 10,
            presets_minutes: [0, 1, 3, 10, u64::MAX],
            ..Preferences::default()
        }
        .sanitized();

        assert_eq!(preferences.alarm_volume, 100);
        assert_eq!(preferences.alarm_repeat_ms, MIN_ALARM_REPEAT_MS);
        assert_eq!(preferences.presets_minutes[0], 1);
        assert_eq!(preferences.presets_minutes[4], MAX_PRESET_MINUTES);
    }
}
'''
Path("src/settings.rs").write_text(settings, encoding="utf-8")


dial = r'''use crate::{app::Mode, settings::DialStyle};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Stroke, Vec2};
use std::{
    f32::consts::{FRAC_PI_2, TAU},
    time::Duration,
};

pub const PAPER: Color32 = Color32::from_rgb(248, 246, 240);
pub const INK: Color32 = Color32::from_rgb(35, 39, 43);
pub const MUTED: Color32 = Color32::from_rgb(125, 127, 126);
pub const ACCENT: Color32 = Color32::from_rgb(160, 52, 47);

const NAVY: Color32 = Color32::from_rgb(7, 27, 52);
const NAVY_SUBDIAL: Color32 = Color32::from_rgb(10, 39, 72);
const NAVY_BEZEL: Color32 = Color32::from_rgb(14, 18, 25);
const NAVY_MUTED: Color32 = Color32::from_rgb(157, 177, 199);
const NAVY_FINE_TICK: Color32 = Color32::from_rgb(91, 119, 148);
const RACING_YELLOW: Color32 = Color32::from_rgb(247, 196, 36);
const RACING_RED: Color32 = Color32::from_rgb(222, 55, 48);

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

fn draw_navy_subdial_bezel(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    scale: f32,
) {
    painter.circle_stroke(
        center,
        radius * 1.16,
        Stroke::new(1.2 * scale, NAVY_BEZEL),
    );

    // Motorsport-inspired railway bezel. Five batons of one colour form a
    // sector, then the colour changes for the next sector.
    for index in 0..60 {
        let color = if (index / 5) % 2 == 0 {
            RACING_YELLOW
        } else {
            RACING_RED
        };
        painter.line_segment(
            [
                radial(center, radius * 1.025, index as f64 / 60.0),
                radial(center, radius * 1.14, index as f64 / 60.0),
            ],
            Stroke::new(3.2 * scale, color),
        );
    }
}

pub fn draw_dial(
    ui: &mut egui::Ui,
    size: f32,
    duration: Duration,
    mode: Mode,
    style: DialStyle,
) {
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
    painter.circle_filled(center, radius * 1.025, palette.face);
    painter.circle_stroke(
        center,
        radius * 0.985,
        Stroke::new(1.0_f32, palette.muted),
    );

    // Alternating railway-style outer bezel.
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
        painter.circle_stroke(
            sub_center,
            sub_radius,
            Stroke::new(1.0_f32, palette.muted),
        );
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
'''
Path("src/dial.rs").write_text(dial, encoding="utf-8")

# Localised labels for appearance controls.
i18n = Path("src/i18n.rs")
text = i18n.read_text(encoding="utf-8")
anchor = '''            (Self::French, "sound") => "Son",\n            (Self::English, "sound") => "Sound",\n'''
insert = anchor + '''            (Self::French, "appearance") => "Apparence",\n            (Self::English, "appearance") => "Appearance",\n            (Self::French, "light_mode") => "Clair",\n            (Self::English, "light_mode") => "Light",\n            (Self::French, "dark_mode") => "Sombre",\n            (Self::English, "dark_mode") => "Dark",\n            (Self::French, "dial_style") => "Cadran",\n            (Self::English, "dial_style") => "Dial",\n            (Self::French, "dial_classic") => "Ivoire classique",\n            (Self::English, "dial_classic") => "Classic ivory",\n            (Self::French, "dial_navy") => "Bleu marine",\n            (Self::English, "dial_navy") => "Navy blue",\n'''
if text.count(anchor) != 1:
    raise SystemExit("i18n appearance anchor not found exactly once")
i18n.write_text(text.replace(anchor, insert, 1), encoding="utf-8")

# Application theme state and controls.
app = Path("src/app.rs")
text = app.read_text(encoding="utf-8")

old = '''use crate::settings::{\n    split_duration, AlarmTone, Preferences, SavedMode, DEFAULT_PRESETS_MINUTES,\n    MAX_ALARM_REPEAT_MS, MIN_ALARM_REPEAT_MS,\n};\n'''
new = '''use crate::settings::{\n    split_duration, AlarmTone, DialStyle, Preferences, SavedMode, UiTheme,\n    DEFAULT_PRESETS_MINUTES, MAX_ALARM_REPEAT_MS, MIN_ALARM_REPEAT_MS,\n};\n'''
if text.count(old) != 1:
    raise SystemExit("settings import anchor failed")
text = text.replace(old, new, 1)

imports_anchor = '''use std::time::{Duration, Instant};\n\n'''
theme_helpers = '''use std::time::{Duration, Instant};\n\nconst DARK_PANEL: Color32 = Color32::from_rgb(16, 21, 29);\nconst DARK_EXTREME: Color32 = Color32::from_rgb(9, 13, 19);\nconst DARK_FAINT: Color32 = Color32::from_rgb(24, 31, 42);\nconst DARK_TEXT: Color32 = Color32::from_rgb(235, 239, 245);\nconst DARK_MUTED: Color32 = Color32::from_rgb(163, 174, 188);\n\nfn theme_text(theme: UiTheme) -> Color32 {\n    match theme {\n        UiTheme::Light => INK,\n        UiTheme::Dark => DARK_TEXT,\n    }\n}\n\nfn theme_muted(theme: UiTheme) -> Color32 {\n    match theme {\n        UiTheme::Light => MUTED,\n        UiTheme::Dark => DARK_MUTED,\n    }\n}\n\nfn apply_ui_theme(ctx: &egui::Context, theme: UiTheme) {\n    let mut visuals = match theme {\n        UiTheme::Light => egui::Visuals::light(),\n        UiTheme::Dark => egui::Visuals::dark(),\n    };\n    visuals.override_text_color = Some(theme_text(theme));\n    visuals.panel_fill = match theme {\n        UiTheme::Light => PAPER,\n        UiTheme::Dark => DARK_PANEL,\n    };\n    visuals.window_fill = visuals.panel_fill;\n    if theme == UiTheme::Dark {\n        visuals.extreme_bg_color = DARK_EXTREME;\n        visuals.faint_bg_color = DARK_FAINT;\n    }\n    visuals.selection.bg_fill = ACCENT;\n    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);\n    ctx.set_visuals(visuals);\n}\n\n'''
if text.count(imports_anchor) != 1:
    raise SystemExit("theme helper anchor failed")
text = text.replace(imports_anchor, theme_helpers, 1)

replace_fields = '''    language: Language,\n    presets_minutes: [u64; 5],\n    export_message: Option<String>,\n'''
new_fields = '''    language: Language,\n    presets_minutes: [u64; 5],\n    ui_theme: UiTheme,\n    dial_style: DialStyle,\n    export_message: Option<String>,\n'''
if text.count(replace_fields) != 1:
    raise SystemExit("state fields anchor failed")
text = text.replace(replace_fields, new_fields, 1)

old_new = '''    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {\n        let mut visuals = egui::Visuals::light();\n        visuals.override_text_color = Some(INK);\n        visuals.panel_fill = PAPER;\n        visuals.selection.bg_fill = ACCENT;\n        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);\n        cc.egui_ctx.set_visuals(visuals);\n\n        let mut style = (*cc.egui_ctx.style()).clone();\n        style.spacing.item_spacing = Vec2::new(10.0, 10.0);\n        style.spacing.button_padding = Vec2::new(15.0, 10.0);\n        cc.egui_ctx.set_style(style);\n\n        let preferences = Preferences::load(cc.storage);\n'''
new_new = '''    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {\n        let preferences = Preferences::load(cc.storage);\n        apply_ui_theme(&cc.egui_ctx, preferences.ui_theme);\n\n        let mut style = (*cc.egui_ctx.style()).clone();\n        style.spacing.item_spacing = Vec2::new(10.0, 10.0);\n        style.spacing.button_padding = Vec2::new(15.0, 10.0);\n        cc.egui_ctx.set_style(style);\n\n'''
if text.count(old_new) != 1:
    raise SystemExit("constructor visuals anchor failed")
text = text.replace(old_new, new_new, 1)

old_assign = '''            language: preferences.language,\n            presets_minutes: preferences.presets_minutes,\n            export_message: None,\n'''
new_assign = '''            language: preferences.language,\n            presets_minutes: preferences.presets_minutes,\n            ui_theme: preferences.ui_theme,\n            dial_style: preferences.dial_style,\n            export_message: None,\n'''
if text.count(old_assign) != 1:
    raise SystemExit("constructor state anchor failed")
text = text.replace(old_assign, new_assign, 1)

old_preferences = '''            language: self.language,\n            presets_minutes: self.presets_minutes,\n        }\n    }\n\n    fn set_language'''
new_preferences = '''            language: self.language,\n            presets_minutes: self.presets_minutes,\n            ui_theme: self.ui_theme,\n            dial_style: self.dial_style,\n        }\n    }\n\n    fn set_ui_theme(&mut self, ctx: &egui::Context, theme: UiTheme) {\n        self.ui_theme = theme;\n        apply_ui_theme(ctx, theme);\n    }\n\n    fn set_language'''
if text.count(old_preferences) != 1:
    raise SystemExit("preferences anchor failed")
text = text.replace(old_preferences, new_preferences, 1)

# Make explicit secondary text readable in both themes.
text = text.replace('.color(MUTED)', '.color(theme_muted(self.ui_theme))')

compact_color = '''        let time_color = if self.mode == Mode::Countdown && self.finished {\n            ACCENT\n        } else {\n            INK\n        };\n'''
compact_new = '''        let time_color = if self.mode == Mode::Countdown && self.finished {\n            ACCENT\n        } else {\n            theme_text(self.ui_theme)\n        };\n'''
if text.count(compact_color) != 1:
    raise SystemExit("compact time colour anchor failed")
text = text.replace(compact_color, compact_new, 1)

main_color = '''                            .color(if self.mode == Mode::Countdown && self.finished {\n                                ACCENT\n                            } else {\n                                INK\n                            }),\n'''
main_color_new = '''                            .color(if self.mode == Mode::Countdown && self.finished {\n                                ACCENT\n                            } else {\n                                theme_text(self.ui_theme)\n                            }),\n'''
if text.count(main_color) != 1:
    raise SystemExit("main time colour anchor failed")
text = text.replace(main_color, main_color_new, 1)

language_block = '''                ui.horizontal(|ui| {\n                    for option in Language::ALL {\n                        if ui\n                            .selectable_value(&mut self.language, option, option.short_label())\n                            .changed()\n                        {\n                            self.set_language(ctx, option);\n                        }\n                    }\n                });\n                ui.add_space(10.0);\n\n                ui.horizontal_wrapped(|ui| {\n'''
appearance_block = '''                ui.horizontal(|ui| {\n                    for option in Language::ALL {\n                        if ui\n                            .selectable_value(&mut self.language, option, option.short_label())\n                            .changed()\n                        {\n                            self.set_language(ctx, option);\n                        }\n                    }\n                });\n                ui.add_space(6.0);\n\n                let mut selected_theme = self.ui_theme;\n                let mut selected_dial = self.dial_style;\n                ui.horizontal_wrapped(|ui| {\n                    ui.label(format!("{} :", language.tr("appearance")));\n                    egui::ComboBox::from_id_salt("ui_theme")\n                        .selected_text(selected_theme.label(language))\n                        .show_ui(ui, |ui| {\n                            for theme in UiTheme::ALL {\n                                ui.selectable_value(\n                                    &mut selected_theme,\n                                    theme,\n                                    theme.label(language),\n                                );\n                            }\n                        });\n                    ui.separator();\n                    ui.label(format!("{} :", language.tr("dial_style")));\n                    egui::ComboBox::from_id_salt("dial_style")\n                        .selected_text(selected_dial.label(language))\n                        .show_ui(ui, |ui| {\n                            for style in DialStyle::ALL {\n                                ui.selectable_value(\n                                    &mut selected_dial,\n                                    style,\n                                    style.label(language),\n                                );\n                            }\n                        });\n                });\n                if selected_theme != self.ui_theme {\n                    self.set_ui_theme(ctx, selected_theme);\n                }\n                self.dial_style = selected_dial;\n                ui.add_space(10.0);\n\n                ui.horizontal_wrapped(|ui| {\n'''
if text.count(language_block) != 1:
    raise SystemExit("language UI block anchor failed")
text = text.replace(language_block, appearance_block, 1)

old_dial_call = '                    draw_dial(ui, size, displayed, self.mode);\n'
new_dial_call = '                    draw_dial(ui, size, displayed, self.mode, self.dial_style);\n'
if text.count(old_dial_call) != 1:
    raise SystemExit("draw_dial call anchor failed")
text = text.replace(old_dial_call, new_dial_call, 1)

app.write_text(text, encoding="utf-8")

# Semantic version 0.2.0 (the conventional spelling of "0.20").
cargo = Path("Cargo.toml")
text = cargo.read_text(encoding="utf-8")
if text.count('version = "0.1.0"') != 1:
    raise SystemExit("Cargo.toml version anchor failed")
cargo.write_text(text.replace('version = "0.1.0"', 'version = "0.2.0"', 1), encoding="utf-8")

lock = Path("Cargo.lock")
text = lock.read_text(encoding="utf-8")
anchor = 'name = "calibre-60"\nversion = "0.1.0"'
if text.count(anchor) != 1:
    raise SystemExit("Cargo.lock root version anchor failed")
lock.write_text(text.replace(anchor, 'name = "calibre-60"\nversion = "0.2.0"', 1), encoding="utf-8")

# Changelog and user-facing documentation.
changelog = Path("CHANGELOG.md")
text = changelog.read_text(encoding="utf-8")
anchor = '## Unreleased\n\n'
addition = '''## Unreleased\n\n### Added for 0.2.0\n\n- Persistent light and dark application themes.\n- Persistent classic ivory and navy-blue dial styles.\n- Navy dial rendered entirely in code, with a yellow/red railway-style baton bezel around the 30-minute subdial.\n- Theme-aware text and secondary colours for comfortable contrast in dark mode.\n\n'''
if text.count(anchor) != 1:
    raise SystemExit("CHANGELOG anchor failed")
changelog.write_text(text.replace(anchor, addition, 1), encoding="utf-8")

readme = Path("README.md")
text = readme.read_text(encoding="utf-8")
text = text.replace(
    'Calibre 60 combines an ivory dial, a sweeping seconds hand, a 30-minute subdial,\nand a digital hours/minutes/seconds/milliseconds display.',
    'Calibre 60 combines code-rendered classic ivory and navy-blue dials, a sweeping seconds hand,\na 30-minute subdial, and a digital hours/minutes/seconds/milliseconds display.',
    1,
)
text = text.replace(
    '- Compact mode for keeping the timer and essential controls in a small corner of the screen.\n',
    '- Compact mode for keeping the timer and essential controls in a small corner of the screen.\n- Persistent light/dark application themes and classic ivory/navy-blue dial styles.\n- Navy dial with a code-rendered yellow/red baton bezel around the 30-minute subdial; no raster dial assets.\n',
    1,
)
readme.write_text(text, encoding="utf-8")

readme_fr = Path("README.fr.md")
text = readme_fr.read_text(encoding="utf-8")
text = text.replace(
    "Calibre 60 associe un cadran ivoire, une aiguille fluide des secondes, un petit\ncompteur de 30 minutes et un affichage numérique heures/minutes/secondes/millisecondes.",
    "Calibre 60 propose des cadrans ivoire classique et bleu marine entièrement dessinés en code,\nune aiguille fluide des secondes, un petit compteur de 30 minutes et un affichage numérique\nheures/minutes/secondes/millisecondes.",
    1,
)
text = text.replace(
    "- Mode compact pour garder le temps et les commandes essentielles dans un coin de l'écran.\n",
    "- Mode compact pour garder le temps et les commandes essentielles dans un coin de l'écran.\n- Thèmes d'interface clair/sombre et cadrans ivoire classique/bleu marine, tous persistants.\n- Cadran bleu marine avec bordure du compteur 30 minutes en bâtonnets jaunes et rouges, dessinée entièrement en code sans image matricielle.\n",
    1,
)
readme_fr.write_text(text, encoding="utf-8")

# The helper is one-shot; the workflow file is removed separately after the bot commit.
Path("scripts/apply_v020_themes.py").unlink()
