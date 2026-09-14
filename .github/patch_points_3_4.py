from pathlib import Path

path = Path("src/main.rs")
source = path.read_text(encoding="utf-8")


def replace_once(old: str, new: str) -> None:
    global source
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"Expected one occurrence, found {count}: {old[:80]!r}")
    source = source.replace(old, new, 1)


replace_once(
    "mod dial;\nmod settings;\n",
    "mod dial;\nmod notifications;\nmod settings;\n#[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\nmod tray;\n",
)

replace_once(
    "use std::time::{Duration, Instant};\n",
    "use std::time::{Duration, Instant};\n#[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\nuse tray::{SystemTray, TrayAction};\n",
)

replace_once(
    "    audio_error: Option<String>,\n}",
    "    audio_error: Option<String>,\n    #[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\n    tray: Option<SystemTray>,\n    tray_error: Option<String>,\n    backgrounded: bool,\n    quit_requested: bool,\n}",
)

replace_once(
    "        alarm.send(AudioCommand::Sound(preferences.sound));\n\n        Self {\n",
    "        alarm.send(AudioCommand::Sound(preferences.sound));\n\n        #[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\n        let (tray, tray_error) = match SystemTray::new(cc.egui_ctx.clone()) {\n            Ok(tray) => (Some(tray), None),\n            Err(error) => (None, Some(error)),\n        };\n        #[cfg(not(any(target_os = \"linux\", target_os = \"windows\")))]\n        let tray_error = Some(String::from(\n            \"La zone de notification n’est pas disponible sur cette plateforme.\",\n        ));\n\n        Self {\n",
)

replace_once(
    "            alarm,\n            audio_error: None,\n        }\n    }\n",
    "            alarm,\n            audio_error: None,\n            #[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\n            tray,\n            tray_error,\n            backgrounded: false,\n            quit_requested: false,\n        }\n    }\n",
)

replace_once(
    "    fn lap_list(&self, ui: &mut egui::Ui, ctx: &egui::Context) {\n",
    '''    fn send_to_background(&mut self, ctx: &egui::Context) {
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

    fn lap_list(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
''',
)

replace_once(
    "    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {\n        for error in self.alarm.errors.try_iter() {\n",
    '''    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
''',
)

replace_once(
    '''                    ui.label(
                        RichText::new(
                            "Espace : marche / pause · L : tour · R : remise à zéro à l’arrêt · Échap : alarme",
                        )
                        .small()
                        .color(MUTED),
                    );

                    if self.mode == Mode::Stopwatch {
''',
    '''                    ui.label(
                        RichText::new(
                            "Espace : marche / pause · L : tour · R : remise à zéro à l’arrêt · Échap : alarme",
                        )
                        .small()
                        .color(MUTED),
                    );

                    #[cfg(any(target_os = "linux", target_os = "windows"))]
                    if self.tray.is_some() {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Réduire en arrière-plan").clicked() {
                                self.send_to_background(ctx);
                            }
                            ui.label(
                                RichText::new(
                                    "Si un minuteur tourne, fermer la fenêtre le laisse actif dans la zone de notification.",
                                )
                                .small()
                                .color(MUTED),
                            );
                        });
                    }

                    if let Some(error) = &self.tray_error {
                        ui.label(RichText::new(error).small().color(MUTED));
                    }

                    if self.mode == Mode::Stopwatch {
''',
)

replace_once(
    '''        if self.stopwatch.running() || self.countdown.running() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
''',
    '''        if self.stopwatch.running() || self.countdown.running() {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.backgrounded {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
''',
)

path.write_text(source, encoding="utf-8")
