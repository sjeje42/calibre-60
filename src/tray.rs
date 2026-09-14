use crate::i18n::Language;
use eframe::egui;
use std::sync::mpsc::{self, Receiver};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    Toggle,
    Quit,
}

pub struct SystemTray {
    _icon: TrayIcon,
    actions: Receiver<TrayAction>,
}

impl SystemTray {
    pub fn new(ctx: egui::Context, language: Language) -> Result<Self, String> {
        let menu = Menu::new();
        let open_item = MenuItem::new(language.tr("tray_open"), true, None);
        let toggle_item = MenuItem::new(language.tr("tray_toggle"), true, None);
        let quit_item = MenuItem::new(language.tr("tray_quit"), true, None);

        menu.append(&open_item)
            .map_err(|error| menu_error(language, error))?;
        menu.append(&toggle_item)
            .map_err(|error| menu_error(language, error))?;
        menu.append(&quit_item)
            .map_err(|error| menu_error(language, error))?;

        let icon = build_icon(language)?;
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Calibre 60 — JérômeLab")
            .with_icon(icon)
            .build()
            .map_err(|error| match language {
                Language::French => format!("Zone de notification indisponible : {error}"),
                Language::English => format!("System tray unavailable: {error}"),
            })?;

        let open_id = open_item.id().clone();
        let toggle_id = toggle_item.id().clone();
        let quit_id = quit_item.id().clone();

        let (sender, actions) = mpsc::channel();

        let menu_sender = sender.clone();
        let menu_ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == open_id {
                Some(TrayAction::Open)
            } else if event.id == toggle_id {
                Some(TrayAction::Toggle)
            } else if event.id == quit_id {
                Some(TrayAction::Quit)
            } else {
                None
            };

            if let Some(action) = action {
                let _ = menu_sender.send(action);
                menu_ctx.request_repaint();
            }
        }));

        let tray_sender = sender;
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            let open = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );

            if open {
                let _ = tray_sender.send(TrayAction::Open);
                ctx.request_repaint();
            }
        }));

        Ok(Self {
            _icon: tray_icon,
            actions,
        })
    }

    pub fn try_action(&self) -> Option<TrayAction> {
        self.actions.try_recv().ok()
    }
}

fn menu_error(language: Language, error: impl std::fmt::Display) -> String {
    match language {
        Language::French => format!("Menu de la zone de notification indisponible : {error}"),
        Language::English => format!("System tray menu unavailable: {error}"),
    }
}

fn build_icon(language: Language) -> Result<Icon, String> {
    const SIZE: u32 = 32;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    let center = (SIZE as f32 - 1.0) / 2.0;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let index = ((y * SIZE + x) * 4) as usize;

            if distance <= 14.0 {
                rgba[index] = 244;
                rgba[index + 1] = 238;
                rgba[index + 2] = 220;
                rgba[index + 3] = 255;

                if distance >= 12.4 {
                    rgba[index] = 42;
                    rgba[index + 1] = 37;
                    rgba[index + 2] = 32;
                }

                if (x == 16 && (7..=17).contains(&y))
                    || (y == 16 && (16..=23).contains(&x))
                {
                    rgba[index] = 178;
                    rgba[index + 1] = 58;
                    rgba[index + 2] = 47;
                }
            }
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).map_err(|error| match language {
        Language::French => format!("Icône de la zone de notification invalide : {error}"),
        Language::English => format!("Invalid system tray icon: {error}"),
    })
}
