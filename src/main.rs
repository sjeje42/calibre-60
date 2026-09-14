#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod alarm;
mod clock;
mod dial;
mod i18n;
mod laps;
mod notifications;
mod settings;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod tray;

use alarm::{Alarm, AlarmConfig, AudioCommand};
use clock::{format_time, Clock};
use dial::{draw_dial, ACCENT, INK, MUTED, PAPER};
use i18n::Language;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use settings::{
    split_duration, AlarmTone, Preferences, SavedMode, DEFAULT_PRESETS_MINUTES,
    MAX_ALARM_REPEAT_MS, MIN_ALARM_REPEAT_MS,
};
use std::time::{Duration, Instant};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use tray::{SystemTray, TrayAction};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Stopwatch,
    Countdown,
}

impl From<SavedMode> for Mode {
    fn from(mode: SavedMode) -> Self {
        match mode {
            SavedMode::Stopwatch => Self::Stopwatch,
            SavedMode::Countdown => Self::Countdown,
        }
    }
}

impl From<Mode> for SavedMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Stopwatch => Self::Stopwatch,
            Mode::Countdown => Self::Countdown,
        }
    }
}

struct Calibre60 {
    mode: Mode,
    stopwatch: Clock,
    countdown: Clock,
    target: Duration,
    duration_fields: [u64; 3],
    laps: Vec<Duration>,
    finished: bool,
    sound: bool,
    alarm_tone: AlarmTone,
    alarm_volume: u8,
    alarm_repeat_ms: u64,
    language: Language,
    presets_minutes: [u64; 5],
    export_message: Option<String>,
    alarm: Alarm,
    audio_error: Option<String>,
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    tray: Option<SystemTray>,
    tray_error: Option<String>,
    backgrounded: bool,
    quit_requested: bool,
    compact_mode: bool,
    normal_window_size: Option<Vec2>,
}

impl Calibre60 {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.override_text_color = Some(INK);
        visuals.panel_fill = PAPER;
        visuals.selection.bg_fill = ACCENT;
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
        cc.egui_ctx.set_visuals(visuals);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = Vec2::new(15.0, 10.0);
        cc.egui_ctx.set_style(style);

        let preferences = Preferences::load(cc.storage);
        let alarm = Alarm::new(cc.egui_ctx.clone(), preferences.language);
        alarm.send(AudioCommand::Configure(AlarmConfig::new(
            preferences.alarm_tone,
            preferences.alarm_volume,
            preferences.alarm_repeat_ms,
        )));
        alarm.send(AudioCommand::Sound(preferences.sound));

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let (tray, tray_error) = match SystemTray::new(cc.egui_ctx.clone(), preferences.language) {
            Ok(tray) => (Some(tray), None),
            Err(error) => (None, Some(error)),
        };
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let tray_error = Some(String::from(preferences.language.tr("tray_unavailable")));

        Self {
            mode: preferences.mode.into(),
            stopwatch: Clock::default(),
            countdown: Clock::default(),
            target: Duration::from_secs(preferences.countdown_seconds),
            duration_fields: split_duration(preferences.countdown_seconds),
            laps: Vec::new(),
            finished: false,
            sound: preferences.sound,
            alarm_tone: preferences.alarm_tone,
            alarm_volume: preferences.alarm_volume,
            alarm_repeat_ms: preferences.alarm_repeat_ms,
            language: preferences.language,
            presets_minutes: preferences.presets_minutes,
            export_message: None,
            alarm,
            audio_error: None,
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            tray,
            tray_error,
            backgrounded: false,
            quit_requested: false,
            compact_mode: false,
            normal_window_size: None,
        }
    }

    fn preferences(&self) -> Preferences {
        Preferences {
            mode: self.mode.into(),
            sound: self.sound,
            countdown_seconds: self.target.as_secs(),
            alarm_tone: self.alarm_tone,
            alarm_volume: self.alarm_volume,
            alarm_repeat_ms: self.alarm_repeat_ms,
            language: self.language,
            presets_minutes: self.presets_minutes,
        }
    }

    fn set_language(&mut self, ctx: &egui::Context, language: Language) {
        self.language = language;
        self.alarm.send(AudioCommand::Language(language));
        self.audio_error = None;

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            match SystemTray::new(ctx.clone(), language) {
                Ok(tray) => {
                    self.tray = Some(tray);
                    self.tray_error = None;
                }
                Err(error) => {
                    self.tray = None;
                    self.tray_error = Some(error);
                }
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            self.tray_error = Some(String::from(language.tr("tray_unavailable")));
        }
    }

    fn sync_alarm_config(&self) {
        self.alarm.send(AudioCommand::Configure(AlarmConfig::new(
            self.alarm_tone,
            self.alarm_volume,
            self.alarm_repeat_ms,
        )));
    }

    fn running(&self) -> bool {
        match self.mode {
            Mode::Stopwatch => self.stopwatch.running(),
            Mode::Countdown => self.countdown.running(),
        }
    }

    fn check_expiration(&mut self, now: Instant) {
        if self.countdown.running() && self.countdown.elapsed(now) >= self.target {
            self.countdown.finish_at(self.target);
            self.finished = true;
            // The independent audio worker already has the same deadline.
        }
    }

    fn acknowledge(&mut self) {
        self.finished = false;
        self.alarm.send(AudioCommand::Cancel);
    }

    fn toggle(&mut self) {
        let now = Instant::now();
        self.check_expiration(now);

        match self.mode {
            Mode::Stopwatch => {
                if self.stopwatch.running() {
                    self.stopwatch.pause(now);
                } else {
                    self.stopwatch.start(now);
                }
            }
            Mode::Countdown => {
                if self.countdown.running() {
                    self.countdown.pause(now);
                    self.alarm.send(AudioCommand::Cancel);
                } else if !self.target.is_zero() {
                    self.acknowledge();
                    if self.countdown.elapsed(now) >= self.target {
                        self.countdown.reset();
                    }
                    self.countdown.start(now);
                    if let Some(end) = self.countdown.deadline(self.target) {
                        self.alarm.send(AudioCommand::Arm(end));
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        if self.running() {
            return;
        }
        match self.mode {
            Mode::Stopwatch => {
                self.stopwatch.reset();
                self.laps.clear();
            }
            Mode::Countdown => {
                self.countdown.reset();
                self.acknowledge();
            }
        }
    }

    fn lap(&mut self) {
        if self.mode == Mode::Stopwatch && self.stopwatch.running() {
            self.laps.push(self.stopwatch.elapsed(Instant::now()));
        }
    }

    fn apply_duration(&mut self) {
        if self.countdown.running() {
            return;
        }
        let [hours, minutes, seconds] = self.duration_fields;
        let total = hours.min(99) * 3600 + minutes.min(59) * 60 + seconds.min(59);
        self.target = Duration::from_secs(total);
        self.countdown.reset();
        self.acknowledge();
    }

    fn copy_laps(&self, ctx: &egui::Context) {
        ctx.copy_text(laps::csv(&self.laps, self.language));
    }

    fn duration_controls(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        ui.add_enabled_ui(!self.countdown.running(), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(language.tr("duration"));
                for (index, suffix, maximum) in [
                    (0, " h", 99_u64),
                    (1, " min", 59_u64),
                    (2, " s", 59_u64),
                ] {
                    ui.add(
                        egui::DragValue::new(&mut self.duration_fields[index])
                            .range(0..=maximum)
                            .suffix(suffix),
                    );
                }
                if ui.button(language.tr("apply")).clicked() {
                    self.apply_duration();
                }
            });

            let presets = self.presets_minutes;
            ui.horizontal_wrapped(|ui| {
                for minutes in presets {
                    let label = if minutes < 60 {
                        format!("{minutes} min")
                    } else if minutes % 60 == 0 {
                        format!("{} h", minutes / 60)
                    } else {
                        format!("{} h {:02}", minutes / 60, minutes % 60)
                    };
                    if ui.button(label).clicked() {
                        self.duration_fields = [minutes / 60, minutes % 60, 0];
                        self.apply_duration();
                    }
                }
            });

            egui::CollapsingHeader::new(language.tr("custom_presets"))
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (index, preset) in self.presets_minutes.iter_mut().enumerate() {
                            ui.label(format!("{} {}", language.tr("preset"), index + 1));
                            ui.add(
                                egui::DragValue::new(preset)
                                    .range(1..=5_999_u64)
                                    .suffix(format!(" {}", language.tr("minutes_short"))),
                            );
                        }
                    });
                    if ui.button(language.tr("restore_presets")).clicked() {
                        self.presets_minutes = DEFAULT_PRESETS_MINUTES;
                    }
                    ui.label(
                        RichText::new(language.tr("presets_autosaved"))
                            .small()
                            .color(MUTED),
                    );
                });
        });

        ui.label(
            RichText::new(format!(
                "{} : {}",
                language.tr("applied_duration"),
                format_time(self.target)
            ))
            .small()
            .color(MUTED),
        );
    }

    fn alarm_controls(&mut self, ui: &mut egui::Ui) {
        let language = self.language;
        egui::CollapsingHeader::new(language.tr("alarm_settings"))
            .default_open(false)
            .show(ui, |ui| {
                let mut changed = false;

                ui.horizontal_wrapped(|ui| {
                    ui.label(language.tr("ringtone"));
                    egui::ComboBox::from_id_salt("alarm_tone")
                        .selected_text(self.alarm_tone.label(language))
                        .show_ui(ui, |ui| {
                            for tone in AlarmTone::ALL {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.alarm_tone,
                                        tone,
                                        tone.label(language),
                                    )
                                    .changed();
                            }
                        });
                });

                let mut volume = u32::from(self.alarm_volume);
                if ui
                    .add(
                        egui::Slider::new(&mut volume, 0..=100)
                            .text(language.tr("volume"))
                            .suffix(" %"),
                    )
                    .changed()
                {
                    self.alarm_volume = volume as u8;
                    changed = true;
                }

                let mut repeat_seconds = self.alarm_repeat_ms as f64 / 1_000.0;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut repeat_seconds,
                            MIN_ALARM_REPEAT_MS as f64 / 1_000.0
                                ..=MAX_ALARM_REPEAT_MS as f64 / 1_000.0,
                        )
                        .step_by(0.25)
                        .text(language.tr("repeat"))
                        .suffix(" s"),
                    )
                    .changed()
                {
                    self.alarm_repeat_ms = (repeat_seconds * 1_000.0).round() as u64;
                    changed = true;
                }

                if changed {
                    self.sync_alarm_config();
                }

                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(self.sound, egui::Button::new(language.tr("test_sound")))
                        .clicked()
                    {
                        self.alarm.send(AudioCommand::Test);
                    }
                    if !self.sound {
                        ui.label(
                            RichText::new(language.tr("enable_sound_test"))
                                .small()
                                .color(MUTED),
                        );
                    }
                });
            });
    }

    fn send_to_background(&mut self, ctx: &egui::Context) {
        self.backgrounded = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        #[cfg(target_os = "linux")]
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    fn restore_from_background(&mut self, ctx: &egui::Context) {
        self.backgrounded = false;
        #[cfg(target_os = "linux")]
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn handle_tray_actions(&mut self, ctx: &egui::Context) {
        let Some(tray) = self.tray.as_ref() else {
            return;
        };

        let actions: Vec<_> = std::iter::from_fn(|| tray.try_action()).collect();
        for action in actions {
            match action {
                TrayAction::Open => self.restore_from_background(ctx),
                TrayAction::Toggle => self.toggle(),
                TrayAction::Quit => {
                    self.quit_requested = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn set_compact_mode(&mut self, ctx: &egui::Context, compact: bool) {
        if compact == self.compact_mode {
            return;
        }

        if compact {
            self.normal_window_size =
                ctx.input(|input| input.viewport().inner_rect.map(|rect| rect.size()));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(360.0, 90.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(560.0, 110.0)));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(440.0, 620.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                self.normal_window_size
                    .take()
                    .unwrap_or_else(|| Vec2::new(620.0, 900.0)),
            ));
        }

        self.compact_mode = compact;
    }

    fn compact_panel(&mut self, ctx: &egui::Context) {
        let language = self.language;
        let now = Instant::now();
        let displayed = match self.mode {
            Mode::Stopwatch => self.stopwatch.elapsed(now),
            Mode::Countdown => self.countdown.remaining(self.target, now),
        };
        let time_color = if self.mode == Mode::Countdown && self.finished {
            ACCENT
        } else {
            INK
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(if self.mode == Mode::Stopwatch { "⏱" } else { "⏳" })
                        .size(24.0),
                );
                ui.label(
                    RichText::new(format_time(displayed))
                        .monospace()
                        .size(28.0)
                        .color(time_color),
                );

                let action_label = if self.running() {
                    language.tr("pause")
                } else if self.mode == Mode::Countdown
                    && self.countdown.elapsed(Instant::now()) >= self.target
                    && !self.target.is_zero()
                {
                    language.tr("restart")
                } else {
                    language.tr("start_resume")
                };
                let can_start = self.mode == Mode::Stopwatch || !self.target.is_zero();
                if ui
                    .add_enabled(
                        can_start,
                        egui::Button::new(if self.running() { "⏸" } else { "▶" }),
                    )
                    .on_hover_text(action_label)
                    .clicked()
                {
                    self.toggle();
                }

                if self.mode == Mode::Stopwatch
                    && ui
                        .add_enabled(
                            self.stopwatch.running(),
                            egui::Button::new(language.tr("lap")),
                        )
                        .clicked()
                {
                    self.lap();
                }

                if self.finished
                    && ui
                        .button("■")
                        .on_hover_text(language.tr("stop_alarm"))
                        .clicked()
                {
                    self.acknowledge();
                }

                if ui
                    .button(format!("↗ {}", language.tr("normal_view")))
                    .clicked()
                {
                    self.set_compact_mode(ctx, false);
                }
            });
        });
    }

    fn lap_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let language = self.language;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(language.tr("laps_title")).strong());
            if ui
                .add_enabled(!self.laps.is_empty(), egui::Button::new(language.tr("copy_csv")))
                .clicked()
            {
                self.copy_laps(ctx);
                self.export_message = None;
            }
            if ui
                .add_enabled(!self.laps.is_empty(), egui::Button::new(language.tr("export_csv")))
                .clicked()
            {
                self.export_message = Some(match laps::export_csv(&self.laps, language) {
                    Ok(path) => format!("{} : {}", language.tr("export_ok"), path.display()),
                    Err(error) => format!("{} : {error}", language.tr("export_error")),
                });
            }
        });

        if self.laps.is_empty() {
            ui.label(RichText::new(language.tr("lap_hint")).color(MUTED));
            return;
        }

        if let Some(stats) = laps::stats(&self.laps) {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} : {}", language.tr("best"), format_time(stats.best)));
                ui.separator();
                ui.label(format!("{} : {}", language.tr("worst"), format_time(stats.slowest)));
                ui.separator();
                ui.label(format!("{} : {}", language.tr("average"), format_time(stats.average)));
            });
        }

        if let Some(message) = &self.export_message {
            ui.label(RichText::new(message).small().color(MUTED));
        }

        egui::ScrollArea::vertical()
            .id_salt("laps")
            .max_height(180.0)
            .show(ui, |ui| {
                egui::Grid::new("lap_table")
                    .striped(true)
                    .spacing([22.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong(language.tr("lap"));
                        ui.strong(language.tr("lap_duration"));
                        ui.strong(language.tr("cumulative"));
                        ui.end_row();

                        for index in (0..self.laps.len()).rev() {
                            let total = self.laps[index];
                            let previous = if index == 0 {
                                Duration::ZERO
                            } else {
                                self.laps[index - 1]
                            };
                            ui.label(format!("{:02}", index + 1));
                            ui.monospace(format_time(total.saturating_sub(previous)));
                            ui.monospace(format_time(total));
                            ui.end_row();
                        }
                    });
            });
    }

}

impl eframe::App for Calibre60 {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        self.handle_tray_actions(ctx);

        let close_requested = ctx.input(|input| input.viewport().close_requested());
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let can_background = self.tray.is_some();
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let can_background = false;

        if close_requested && !self.quit_requested && self.running() && can_background {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.send_to_background(ctx);
        }

        for error in self.alarm.errors.try_iter() {
            self.audio_error = Some(error);
        }
        self.check_expiration(Instant::now());

        // Do not intercept shortcuts while editing duration fields.
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.finished {
                self.acknowledge();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.toggle();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::L)) {
                self.lap();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                self.reset();
            }
        }

        if self.compact_mode {
            self.compact_panel(ctx);
            if self.stopwatch.running() || self.countdown.running() {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
            return;
        }

        let language = self.language;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("main")
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("CALIBRE 60").size(24.0).strong());
                        ui.label(
                            RichText::new("J É R Ô M E L A B")
                                .size(11.0)
                                .color(MUTED),
                        );
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        for option in Language::ALL {
                            if ui
                                .selectable_value(&mut self.language, option, option.short_label())
                                .changed()
                            {
                                self.set_language(ctx, option);
                            }
                        }
                    });
                    ui.add_space(10.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut self.mode,
                            Mode::Stopwatch,
                            language.tr("stopwatch"),
                        );
                        ui.selectable_value(
                            &mut self.mode,
                            Mode::Countdown,
                            language.tr("countdown"),
                        );
                        if ui.checkbox(&mut self.sound, language.tr("sound")).changed() {
                            self.alarm.send(AudioCommand::Sound(self.sound));
                        }
                    });
                    ui.separator();

                    // Countdown notifications remain visible in either tab.
                    if self.finished {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(language.tr("time_up"))
                                    .strong()
                                    .color(ACCENT),
                            );
                            if ui.button(language.tr("stop_alarm")).clicked() {
                                self.acknowledge();
                            }
                        });
                    }

                    if self.mode == Mode::Countdown {
                        self.duration_controls(ui);
                        self.alarm_controls(ui);
                        ui.separator();
                    }

                    let now = Instant::now();
                    let displayed = match self.mode {
                        Mode::Stopwatch => self.stopwatch.elapsed(now),
                        Mode::Countdown => self.countdown.remaining(self.target, now),
                    };
                    let font_size = (ui.available_width() / 9.5).clamp(26.0, 56.0);

                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(format_time(displayed))
                                .monospace()
                                .size(font_size)
                                .color(
                                    if self.mode == Mode::Countdown && self.finished {
                                        ACCENT
                                    } else {
                                        INK
                                    },
                                ),
                        );
                        let status = if self.running() {
                            language.tr("running")
                        } else if self.mode == Mode::Countdown && self.finished {
                            language.tr("finished")
                        } else {
                            language.tr("stopped")
                        };
                        ui.label(RichText::new(status).size(11.0).color(MUTED));
                        let size = ui.available_width().min(520.0);
                        draw_dial(ui, size, displayed, self.mode);
                    });

                    ui.horizontal_wrapped(|ui| {
                        let label = if self.running() {
                            language.tr("pause")
                        } else if self.mode == Mode::Countdown
                            && self.countdown.elapsed(Instant::now()) >= self.target
                            && !self.target.is_zero()
                        {
                            language.tr("restart")
                        } else {
                            language.tr("start_resume")
                        };
                        let can_start =
                            self.mode == Mode::Stopwatch || !self.target.is_zero();
                        if ui
                            .add_enabled(
                                can_start,
                                egui::Button::new(
                                    RichText::new(label).color(Color32::WHITE),
                                )
                                .fill(ACCENT),
                            )
                            .clicked()
                        {
                            self.toggle();
                        }

                        if self.mode == Mode::Stopwatch
                            && ui
                                .add_enabled(
                                    self.stopwatch.running(),
                                    egui::Button::new(language.tr("lap")),
                                )
                                .clicked()
                        {
                            self.lap();
                        }

                        if ui
                            .add_enabled(
                                !self.running(),
                                egui::Button::new(language.tr("reset")),
                            )
                            .clicked()
                        {
                            self.reset();
                        }
                    });

                    ui.add_space(5.0);
                    ui.label(
                        RichText::new(language.tr("shortcuts"))
                        .small()
                        .color(MUTED),
                    );

                    if ui.button(language.tr("compact_mode")).clicked() {
                        self.set_compact_mode(ctx, true);
                    }

                    #[cfg(any(target_os = "linux", target_os = "windows"))]
                    if self.tray.is_some() {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button(language.tr("background")).clicked() {
                                self.send_to_background(ctx);
                            }
                            ui.label(
                                RichText::new(language.tr("background_info"))
                                .small()
                                .color(MUTED),
                            );
                        });
                    }

                    if let Some(error) = &self.tray_error {
                        ui.label(RichText::new(error).small().color(MUTED));
                    }

                    if self.mode == Mode::Stopwatch {
                        ui.separator();
                        self.lap_list(ui, ctx);
                    }
                    if let Some(error) = &self.audio_error {
                        ui.separator();
                        ui.colored_label(ACCENT, error);
                        ui.small(language.tr("visual_alert"));
                    }
                    ui.add_space(10.0);
                });
        });

        if self.stopwatch.running() || self.countdown.running() {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.backgrounded {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.preferences().save(storage);
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("fr.jeromelab.calibre60")
            .with_inner_size([620.0, 900.0])
            .with_min_inner_size([440.0, 620.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "Calibre 60 — JérômeLab",
        options,
        Box::new(|cc| Ok(Box::new(Calibre60::new(cc)))),
    )
}
