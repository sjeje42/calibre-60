use serde::{Deserialize, Serialize};

pub const MAX_COUNTDOWN_SECONDS: u64 = 99 * 3_600 + 59 * 60 + 59;
pub const MIN_ALARM_REPEAT_MS: u64 = 500;
pub const MAX_ALARM_REPEAT_MS: u64 = 5_000;
const STORAGE_KEY: &str = "calibre60.preferences";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedMode {
    #[default]
    Stopwatch,
    Countdown,
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
    pub const ALL: [Self; 4] = [
        Self::Classic,
        Self::DoubleBeep,
        Self::Chime,
        Self::Digital,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classique",
            Self::DoubleBeep => "Double bip",
            Self::Chime => "Carillon",
            Self::Digital => "Numérique",
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
            ..Preferences::default()
        }
        .sanitized();

        assert_eq!(preferences.alarm_volume, 100);
        assert_eq!(preferences.alarm_repeat_ms, MIN_ALARM_REPEAT_MS);
    }
}
