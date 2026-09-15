# Changelog

All notable changes to Calibre 60 will be documented in this file.

## Unreleased

## 0.1.0 — 2026-09-15

### Added

- Native stopwatch and countdown timer for Linux and Windows.
- Classic analog chronograph dial with digital millisecond display.
- Independent stopwatch and countdown state when switching modes.
- Lap recording with per-lap and cumulative times.
- Lap statistics: best lap, slowest lap, and average lap time.
- Clipboard copy and UTF-8 CSV export for lap data.
- Countdown duration up to 99 h 59 min 59 s.
- Five customizable and persistent countdown presets.
- Four configurable alarm tones with volume and repetition controls.
- Alarm sound preview and global mute toggle.
- Native desktop notification when a countdown finishes.
- Linux/Windows system-tray integration with open, start/pause, and quit actions.
- Background mode that keeps active timers running when the main window is hidden.
- Compact mode with the current time and essential controls in a small window.
- French and English user interface with persistent language selection.
- Persistent application preferences and window state.
- Keyboard shortcuts for start/pause, lap, reset, and alarm acknowledgement.
- Debian desktop entry and application icon.
- Automated Debian 13 `.deb`, Linux x64 `.tar.gz`, and Windows x64 `.zip` release packaging.
- SHA-256 checksum generation for tagged releases.
- Reproducible dependency resolution through a committed `Cargo.lock`.

### Quality and maintenance

- Timing is based on Rust's monotonic `Instant` clock to avoid cumulative repaint drift.
- Alarm playback runs in a separate worker thread from the UI.
- Application state and UI logic were moved from `main.rs` to a dedicated `app.rs` module.
- CI validates `rustfmt`, Clippy with warnings denied, `cargo-audit`, unit tests, and release builds.
- CI covers Debian 13 and Windows x64.
- Source code is licensed under GNU GPL v3.
