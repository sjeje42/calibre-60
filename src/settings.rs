use crate::i18n::Language;
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
