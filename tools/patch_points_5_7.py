from pathlib import Path
import re

path = Path("src/main.rs")
source = path.read_text(encoding="utf-8")


def once(old: str, new: str) -> None:
    global source
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"Expected one occurrence, found {count}: {old[:100]!r}")
    source = source.replace(old, new, 1)


once(
    "mod dial;\nmod notifications;\nmod settings;\n",
    "mod dial;\nmod i18n;\nmod laps;\nmod notifications;\nmod settings;\n",
)
once(
    "use dial::{draw_dial, ACCENT, INK, MUTED, PAPER};\n",
    "use dial::{draw_dial, ACCENT, INK, MUTED, PAPER};\nuse i18n::Language;\n",
)
once(
    "    split_duration, AlarmTone, Preferences, SavedMode, MAX_ALARM_REPEAT_MS,\n    MIN_ALARM_REPEAT_MS,\n",
    "    split_duration, AlarmTone, Preferences, SavedMode, DEFAULT_PRESETS_MINUTES,\n    MAX_ALARM_REPEAT_MS, MIN_ALARM_REPEAT_MS,\n",
)
once(
    "    alarm_repeat_ms: u64,\n    alarm: Alarm,\n",
    "    alarm_repeat_ms: u64,\n    language: Language,\n    presets_minutes: [u64; 5],\n    export_message: Option<String>,\n    alarm: Alarm,\n",
)
once(
    "        let alarm = Alarm::new(cc.egui_ctx.clone());\n",
    "        let alarm = Alarm::new(cc.egui_ctx.clone(), preferences.language);\n",
)
once(
    "        let (tray, tray_error) = match SystemTray::new(cc.egui_ctx.clone()) {\n",
    "        let (tray, tray_error) = match SystemTray::new(cc.egui_ctx.clone(), preferences.language) {\n",
)
once(
    "        let tray_error = Some(String::from(\n            \"La zone de notification n’est pas disponible sur cette plateforme.\",\n        ));\n",
    "        let tray_error = Some(String::from(preferences.language.tr(\"tray_unavailable\")));\n",
)
once(
    "            alarm_repeat_ms: preferences.alarm_repeat_ms,\n            alarm,\n",
    "            alarm_repeat_ms: preferences.alarm_repeat_ms,\n            language: preferences.language,\n            presets_minutes: preferences.presets_minutes,\n            export_message: None,\n            alarm,\n",
)
once(
    "            alarm_repeat_ms: self.alarm_repeat_ms,\n        }\n    }\n\n    fn sync_alarm_config",
    "            alarm_repeat_ms: self.alarm_repeat_ms,\n            language: self.language,\n            presets_minutes: self.presets_minutes,\n        }\n    }\n\n    fn set_language(&mut self, ctx: &egui::Context, language: Language) {\n        self.language = language;\n        self.alarm.send(AudioCommand::Language(language));\n        self.audio_error = None;\n\n        #[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\n        {\n            match SystemTray::new(ctx.clone(), language) {\n                Ok(tray) => {\n                    self.tray = Some(tray);\n                    self.tray_error = None;\n                }\n                Err(error) => {\n                    self.tray = None;\n                    self.tray_error = Some(error);\n                }\n            }\n        }\n        #[cfg(not(any(target_os = \"linux\", target_os = \"windows\")))]\n        {\n            self.tray_error = Some(String::from(language.tr(\"tray_unavailable\")));\n        }\n    }\n\n    fn sync_alarm_config",
)

source, count = re.subn(
    r"    fn copy_laps\(&self, ctx: &egui::Context\) \{.*?\n    \}\n\n    fn duration_controls",
    """    fn copy_laps(&self, ctx: &egui::Context) {
        ctx.copy_text(laps::csv(&self.laps, self.language));
    }

    fn duration_controls""",
    source,
    count=1,
    flags=re.S,
)
if count != 1:
    raise SystemExit("Could not replace copy_laps")

duration_fn = """    fn duration_controls(&mut self, ui: &mut egui::Ui) {
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
"""
source, count = re.subn(
    r"    fn duration_controls\(&mut self, ui: &mut egui::Ui\) \{.*?\n    \}\n\n    fn alarm_controls",
    duration_fn + "\n    fn alarm_controls",
    source,
    count=1,
    flags=re.S,
)
if count != 1:
    raise SystemExit("Could not replace duration_controls")

once(
    "    fn alarm_controls(&mut self, ui: &mut egui::Ui) {\n        egui::CollapsingHeader::new(\"Réglages de l’alarme\")\n",
    "    fn alarm_controls(&mut self, ui: &mut egui::Ui) {\n        let language = self.language;\n        egui::CollapsingHeader::new(language.tr(\"alarm_settings\"))\n",
)
for old, new in {
    'ui.label("Sonnerie :");': 'ui.label(language.tr("ringtone"));',
    '.selected_text(self.alarm_tone.label())': '.selected_text(self.alarm_tone.label(language))',
    'tone.label(),': 'tone.label(language),',
    '.text("Volume")': '.text(language.tr("volume"))',
    '.text("Répétition")': '.text(language.tr("repeat"))',
    'egui::Button::new("Tester le son")': 'egui::Button::new(language.tr("test_sound"))',
    'RichText::new("Active « Son » pour tester.")': 'RichText::new(language.tr("enable_sound_test"))',
}.items():
    if old not in source:
        raise SystemExit(f"Missing alarm replacement: {old}")
    source = source.replace(old, new)

lap_fn = """    fn lap_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
"""
source, count = re.subn(
    r"    fn lap_list\(&self, ui: &mut egui::Ui, ctx: &egui::Context\) \{.*?\n    \}\n\}",
    lap_fn + "\n}",
    source,
    count=1,
    flags=re.S,
)
if count != 1:
    raise SystemExit("Could not replace lap_list")

once(
    "        egui::CentralPanel::default().show(ctx, |ui| {\n",
    "        let language = self.language;\n        egui::CentralPanel::default().show(ctx, |ui| {\n",
)
once(
    """                    ui.add_space(10.0);

                    ui.horizontal_wrapped(|ui| {
""",
    """                    ui.add_space(8.0);
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
""",
)

for old, new in {
    '"Chronomètre",': 'language.tr("stopwatch"),',
    '"Compte à rebours",': 'language.tr("countdown"),',
    'if ui.checkbox(&mut self.sound, "Son").changed()': 'if ui.checkbox(&mut self.sound, language.tr("sound")).changed()',
    'RichText::new("TEMPS ÉCOULÉ")': 'RichText::new(language.tr("time_up"))',
    'if ui.button("Arrêter l’alarme").clicked()': 'if ui.button(language.tr("stop_alarm")).clicked()',
    '"EN COURS"': 'language.tr("running")',
    '"TERMINÉ"': 'language.tr("finished")',
    '"À L’ARRÊT"': 'language.tr("stopped")',
    '"Pause"': 'language.tr("pause")',
    '"Relancer"': 'language.tr("restart")',
    '"Démarrer / Reprendre"': 'language.tr("start_resume")',
    'egui::Button::new("Tour")': 'egui::Button::new(language.tr("lap"))',
    'egui::Button::new("Réinitialiser")': 'egui::Button::new(language.tr("reset"))',
    'RichText::new(\n                            "Espace : marche / pause · L : tour · R : remise à zéro à l’arrêt · Échap : alarme",\n                        )': 'RichText::new(language.tr("shortcuts"))',
    'if ui.button("Réduire en arrière-plan").clicked()': 'if ui.button(language.tr("background")).clicked()',
    'RichText::new(\n                                    "Si un minuteur tourne, fermer la fenêtre le laisse actif dans la zone de notification.",\n                                )': 'RichText::new(language.tr("background_info"))',
    'ui.small("L’alerte visuelle reste disponible.");': 'ui.small(language.tr("visual_alert"));',
}.items():
    if old not in source:
        raise SystemExit(f"Missing UI replacement: {old[:80]!r}")
    source = source.replace(old, new)

path.write_text(source, encoding="utf-8")
