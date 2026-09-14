use serde::{Deserialize, Serialize};

pub const MAX_COUNTDOWN_SECONDS: u64 = 99 * 3_600 + 59 * 60 + 59;
const STORAGE_KEY: &str = "calibre60.preferences";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedMode {
    #[default]
    Stopwatch,
    Countdown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub mode: SavedMode,
    pub sound: bool,
    pub countdown_seconds: u64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            mode: SavedMode::Stopwatch,
            sound: true,
            countdown_seconds: 5 * 60,
        }
    }
}

impl Preferences {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        let mut preferences = storage
            .and_then(|storage| eframe::get_value::<Self>(storage, STORAGE_KEY))
            .unwrap_or_default();
        preferences.countdown_seconds = preferences
            .countdown_seconds
            .min(MAX_COUNTDOWN_SECONDS);
        preferences
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, self);
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
    }

    #[test]
    fn split_duration_clamps_to_the_supported_maximum() {
        assert_eq!(split_duration(3_661), [1, 1, 1]);
        assert_eq!(split_duration(u64::MAX), [99, 59, 59]);
    }
}
