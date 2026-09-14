from pathlib import Path

src = Path("src/main.rs")
text = src.read_text(encoding="utf-8")
state_marker = "#[derive(Clone, Copy, PartialEq)]"
main_marker = "fn main() -> eframe::Result<()> {"
start = text.index(state_marker)
main_start = text.index(main_marker)

body = text[start:main_start]
body = body.replace("enum Mode {", "pub(crate) enum Mode {", 1)
body = body.replace("struct Calibre60 {", "pub(crate) struct Calibre60 {", 1)
body = body.replace("    fn new(cc: &eframe::CreationContext<'_>) -> Self {", "    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {", 1)

preamble = '''use crate::alarm::{Alarm, AlarmConfig, AudioCommand};
use crate::clock::{format_time, Clock};
use crate::dial::{draw_dial, ACCENT, INK, MUTED, PAPER};
use crate::i18n::Language;
use crate::laps;
use crate::settings::{
    split_duration, AlarmTone, Preferences, SavedMode, DEFAULT_PRESETS_MINUTES,
    MAX_ALARM_REPEAT_MS, MIN_ALARM_REPEAT_MS,
};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::tray::{SystemTray, TrayAction};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use std::time::{Duration, Instant};

'''
Path("src/app.rs").write_text(preamble + body, encoding="utf-8")

main_fn = text[main_start:]
new_main = '''#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod alarm;
mod app;
mod clock;
mod dial;
mod i18n;
mod laps;
mod notifications;
mod settings;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod tray;

use app::Calibre60;
use eframe::egui;

''' + main_fn
src.write_text(new_main, encoding="utf-8")

dial = Path("src/dial.rs")
dial_text = dial.read_text(encoding="utf-8").replace("use crate::Mode;", "use crate::app::Mode;", 1)
dial.write_text(dial_text, encoding="utf-8")

Path("scripts/refactor_main.py").unlink()
