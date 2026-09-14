use crate::{clock::format_time, i18n::Language};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LapStats {
    pub best: Duration,
    pub slowest: Duration,
    pub average: Duration,
}

pub fn durations(laps: &[Duration]) -> Vec<Duration> {
    let mut previous = Duration::ZERO;
    laps.iter()
        .map(|&total| {
            let lap = total.saturating_sub(previous);
            previous = total;
            lap
        })
        .collect()
}

pub fn stats(laps: &[Duration]) -> Option<LapStats> {
    let values = durations(laps);
    let first = *values.first()?;
    let mut best = first;
    let mut slowest = first;
    let mut sum = 0.0_f64;

    for value in values {
        best = best.min(value);
        slowest = slowest.max(value);
        sum += value.as_secs_f64();
    }

    Some(LapStats {
        best,
        slowest,
        average: Duration::from_secs_f64(sum / laps.len() as f64),
    })
}

pub fn csv(laps: &[Duration], language: Language) -> String {
    let mut output = String::new();
    output.push('\u{feff}');
    output.push_str(&format!(
        "{};{};{}\n",
        language.tr("lap"),
        language.tr("lap_duration"),
        language.tr("cumulative")
    ));

    let mut previous = Duration::ZERO;
    for (index, &total) in laps.iter().enumerate() {
        output.push_str(&format!(
            "{};{};{}\n",
            index + 1,
            format_time(total.saturating_sub(previous)),
            format_time(total)
        ));
        previous = total;
    }
    output
}

pub fn export_csv(laps: &[Duration], language: Language) -> io::Result<PathBuf> {
    let directory = download_directory().unwrap_or_else(home_directory);
    fs::create_dir_all(&directory)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = directory.join(format!("calibre-60-laps-{timestamp}.csv"));
    fs::write(&path, csv(laps, language))?;
    Ok(path)
}

fn home_directory() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn download_directory() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let home = home_directory();
        let config_home = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        if let Ok(contents) = fs::read_to_string(config_home.join("user-dirs.dirs")) {
            for line in contents.lines() {
                let Some(value) = line.strip_prefix("XDG_DOWNLOAD_DIR=") else {
                    continue;
                };
                let value = value
                    .trim()
                    .trim_matches('"')
                    .replace("$HOME", &home.to_string_lossy());
                if !value.is_empty() {
                    return Some(PathBuf::from(value));
                }
            }
        }
        let fallback = home.join("Downloads");
        if fallback.exists() {
            return Some(fallback);
        }
        let localized = home.join("Téléchargements");
        if localized.exists() {
            return Some(localized);
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        Some(home_directory().join("Downloads"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Some(home_directory().join("Downloads"))
    }
}

#[allow(dead_code)]
fn _is_directory(path: &Path) -> bool {
    path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lap_durations_and_stats_are_correct() {
        let laps = [
            Duration::from_secs(10),
            Duration::from_secs(24),
            Duration::from_secs(36),
        ];
        assert_eq!(
            durations(&laps),
            vec![
                Duration::from_secs(10),
                Duration::from_secs(14),
                Duration::from_secs(12)
            ]
        );
        let stats = stats(&laps).unwrap();
        assert_eq!(stats.best, Duration::from_secs(10));
        assert_eq!(stats.slowest, Duration::from_secs(14));
        assert_eq!(stats.average, Duration::from_secs(12));
    }

    #[test]
    fn csv_uses_selected_language() {
        let laps = [Duration::from_secs(2)];
        let text = csv(&laps, Language::English);
        assert!(text.contains("Lap;Lap time;Cumulative"));
    }
}
