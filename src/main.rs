#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod alarm;
mod clock;
mod dial;
mod settings;

use alarm::{Alarm, AudioCommand};
use clock::{format_time, Clock};
use dial::{draw_dial, ACCENT, INK, MUTED, PAPER};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use settings::{split_duration, Preferences, SavedMode};
use std::time::{Duration, Instant};

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
    alarm: Alarm,
    audio_error: Option<String>,
}

impl Calibre60 {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.override_text_color = Some(INK);
        visuals.panel_fill = PAPER;
        visuals.selection.bg_fill = ACCENT;
        visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        cc.egui_ctx.set_visuals(visuals);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = Vec2::new(15.0, 10.0);
        cc.egui_ctx.set_style(style);

        let preferences = Preferences::load(cc.storage);
        let alarm = Alarm::new(cc.egui_ctx.clone());
        alarm.send(AudioCommand::Sound(preferences.sound));

        Self {
            mode: preferences.mode.into(),
            stopwatch: Clock::default(),
            countdown: Clock::default(),
            target: Duration::from_secs(preferences.countdown_seconds),
            duration_fields: split_duration(preferences.countdown_seconds),
            laps: Vec::new(),
            finished: false,
            sound: preferences.sound,
            alarm,
            audio_error: None,
        }
    }

    fn preferences(&self) -> Preferences {
        Preferences {
            mode: self.mode.into(),
            sound: self.sound,
            countdown_seconds: self.target.as_secs(),
        }
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
        let mut csv = String::from("Tour;Duree du tour;Temps cumule\n");
        let mut previous = Duration::ZERO;
        for (index, &total) in self.laps.iter().enumerate() {
            csv.push_str(&format!(
                "{};{};{}\n",
                index + 1,
                format_time(total.saturating_sub(previous)),
                format_time(total)
            ));
            previous = total;
        }
        ctx.copy_text(csv);
    }

    fn duration_controls(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(!self.countdown.running(), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Durée :");
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
                if ui.button("Appliquer").clicked() {
                    self.apply_duration();
                }
            });

            ui.horizontal_wrapped(|ui| {
                for minutes in [1_u64, 3, 5, 10, 25] {
                    if ui.button(format!("{minutes} min")).clicked() {
                        self.duration_fields = [0, minutes, 0];
                        self.apply_duration();
                    }
                }
            });
        });

        ui.label(
            RichText::new(format!("Durée appliquée : {}", format_time(self.target)))
                .small().color(MUTED),
        );
    }

    fn lap_list(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("TEMPS INTERMÉDIAIRES").strong());
            if ui.add_enabled(
                !self.laps.is_empty(), egui::Button::new("Copier CSV"),
            ).clicked() {
                self.copy_laps(ctx);
            }
        });

        if self.laps.is_empty() {
            ui.label(
                RichText::new("Appuie sur « Tour » pendant le chronométrage.")
                    .color(MUTED),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("laps").max_height(180.0)
            .show(ui, |ui| {
                egui::Grid::new("lap_table")
                    .striped(true).spacing([22.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong("Tour");
                        ui.strong("Durée du tour");
                        ui.strong("Temps cumulé");
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

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt("main").show(ui, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("CALIBRE 60").size(24.0).strong());
                    ui.label(
                        RichText::new("J É R Ô M E L A B")
                            .size(11.0).color(MUTED),
                    );
                });
                ui.add_space(10.0);

                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.mode, Mode::Stopwatch, "Chronomètre");
                    ui.selectable_value(&mut self.mode, Mode::Countdown, "Compte à rebours");
                    if ui.checkbox(&mut self.sound, "Son").changed() {
                        self.alarm.send(AudioCommand::Sound(self.sound));
                    }
                });
                ui.separator();

                // Countdown notifications remain visible in either tab.
                if self.finished {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("TEMPS ÉCOULÉ").strong().color(ACCENT));
                        if ui.button("Arrêter l’alarme").clicked() {
                            self.acknowledge();
                        }
                    });
                }

                if self.mode == Mode::Countdown {
                    self.duration_controls(ui);
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
                            .monospace().size(font_size)
                            .color(if self.mode == Mode::Countdown && self.finished {
                                ACCENT
                            } else { INK }),
                    );
                    let status = if self.running() {
                        "EN COURS"
                    } else if self.mode == Mode::Countdown && self.finished {
                        "TERMINÉ"
                    } else {
                        "À L’ARRÊT"
                    };
                    ui.label(RichText::new(status).size(11.0).color(MUTED));
                    let size = ui.available_width().min(520.0);
                    draw_dial(ui, size, displayed, self.mode);
                });

                ui.horizontal_wrapped(|ui| {
                    let label = if self.running() {
                        "Pause"
                    } else if self.mode == Mode::Countdown
                        && self.countdown.elapsed(Instant::now()) >= self.target
                        && !self.target.is_zero()
                    {
                        "Relancer"
                    } else {
                        "Démarrer / Reprendre"
                    };
                    let can_start = self.mode == Mode::Stopwatch || !self.target.is_zero();
                    if ui.add_enabled(
                        can_start,
                        egui::Button::new(RichText::new(label).color(Color32::WHITE))
                            .fill(ACCENT),
                    ).clicked() {
                        self.toggle();
                    }

                    if self.mode == Mode::Stopwatch
                        && ui.add_enabled(
                            self.stopwatch.running(), egui::Button::new("Tour"),
                        ).clicked()
                    {
                        self.lap();
                    }

                    if ui.add_enabled(
                        !self.running(), egui::Button::new("Réinitialiser"),
                    ).clicked() {
                        self.reset();
                    }
                });

                ui.add_space(5.0);
                ui.label(
                    RichText::new(
                        "Espace : marche / pause · L : tour · R : remise à zéro à l’arrêt · Échap : alarme"
                    ).small().color(MUTED),
                );

                if self.mode == Mode::Stopwatch {
                    ui.separator();
                    self.lap_list(ui, ctx);
                }
                if let Some(error) = &self.audio_error {
                    ui.separator();
                    ui.colored_label(ACCENT, error);
                    ui.small("L’alerte visuelle reste disponible.");
                }
                ui.add_space(10.0);
            });
        });

        if self.stopwatch.running() || self.countdown.running() {
            ctx.request_repaint_after(Duration::from_millis(16));
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
