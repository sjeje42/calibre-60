# Calibre 60

**A native Rust stopwatch and countdown timer with a classic analog chronograph dial.**

English · [Français](README.fr.md)

Calibre 60 combines an ivory dial, a sweeping seconds hand, a 30-minute subdial,
and a digital hours/minutes/seconds/milliseconds display. It is built for Linux
(including Debian 13) and Windows with egui/eframe and rodio.

The application interface can be switched instantly between **French and English**.
The selected language is saved between sessions.

## Features

- Independent stopwatch and countdown: switching tabs does not stop either clock.
- Start, pause, resume, reset, and millisecond display.
- Compact mode for keeping the timer and essential controls in a small corner of the screen.
- Lap duration and cumulative time, newest lap first.
- Lap analytics: best lap, slowest lap, and average lap time.
- Copy lap data to the clipboard as semicolon-separated CSV.
- Export lap data directly to a UTF-8 CSV file in the user's downloads folder.
- Countdown up to 99 h 59 min 59 s.
- Five persistent, customizable countdown presets; defaults are 1, 3, 5, 10, and 25 minutes.
- Four alarm tones, volume control, repetition interval, sound preview, and mute toggle.
- Native desktop notification when a countdown finishes.
- Background/system-tray mode on Linux and Windows, with Open, Start/Pause, and Quit actions.
- A separate alarm thread, independent of window repainting.
- Persistent preferences, including language, presets, alarm settings, and last countdown duration.
- Resizable, high-DPI-aware vector interface with keyboard shortcuts.
- No account or web service required at runtime.

## Status and downloads

The [CI workflow](../../actions/workflows/ci.yml) builds and runs the unit tests on
Debian 13 and Windows. Check the latest run for the actual validation result.

After a successful CI run, development builds remain available from the **Artifacts**
section as `calibre-60-debian13-x64` and `calibre-60-windows-x64`.
Access requires permission to this private repository.

When a `v*` tag is pushed, the release workflow automatically builds a **Debian 13
`.deb` package**, a **Linux x64 `.tar.gz` archive**, and a **Windows x64 `.zip` archive**,
generates **SHA-256 checksums**, and publishes the corresponding GitHub Release.
The Debian package also installs the application icon and desktop-menu entry.

CI validates compilation and automated timing/statistics tests. It does not fully
validate desktop integration such as actual audio hardware, GNOME tray extensions,
notification presentation, or suspend/resume behavior.

## Build on Debian 13

Install the build dependencies:

```bash
sudo apt update
sudo apt install git curl ca-certificates build-essential pkg-config \
  libasound2-dev libxkbcommon-dev libwayland-dev \
  libegl1-mesa-dev libgl1-mesa-dev
```

Install a current stable Rust toolchain using [rustup](https://rustup.rs/).
Restart the terminal afterwards if `cargo` is not found.

Clone the repository using your configured GitHub authentication, then run:

```bash
git clone https://github.com/sjeje42/calibre-60.git
cd calibre-60
cargo test
cargo run --release
```

The executable is `target/release/calibre-60`.
It requires a graphical desktop (X11 or Wayland), working graphics drivers, and
Linux shared libraries; it is not a fully static portable binary.

To add it to your user executable directory after building:

```bash
install -Dm755 target/release/calibre-60 "$HOME/.local/bin/calibre-60"
```

## Build on Windows

1. Install [Rust for Windows](https://rustup.rs/).
2. Install the Microsoft C++ build tools when prompted, including the
   **Desktop development with C++** workload and a Windows SDK.
3. Open a new PowerShell terminal in the cloned repository.

```powershell
cargo test
cargo build --release
.\target\release\calibre-60.exe
```

The executable is `target\release\calibre-60.exe`.
Rust is needed for compilation, not for subsequently running the compiled app.

## Using Calibre 60

Use the **FR / EN** selector below the title to change the interface language.
The choice is persistent and also updates alarm notifications and the system-tray menu.

### Stopwatch

Select **Stopwatch**, then **Start / Resume**. Press **Lap** to record intermediate
times. The lap panel displays the best, slowest, and average lap automatically.

**Copy CSV** copies the table to the clipboard. **Export CSV** saves a UTF-8 CSV file
in the user's downloads directory. The export uses the currently selected interface
language for its column headers.

### Countdown

Select **Countdown**, edit hours/minutes/seconds while stopped, then choose **Apply**.
You can also use one of the five quick presets.

Open **Customize presets** to edit the five preset durations. Values are saved
automatically. **Restore defaults** returns them to 1, 3, 5, 10, and 25 minutes.

When the countdown expires, Calibre 60 displays its visual warning, plays the selected
alarm when sound is enabled, and sends a native desktop notification.

### Compact mode

Choose **Compact mode** to shrink the window to a small bar showing the current timer,
time value, and essential controls. In stopwatch mode, **Lap** remains available while
timing. **Normal view** restores the full chronograph and the previous window size.

### Background mode

On Linux and Windows, **Run in background** sends Calibre 60 to the system tray.
If a timer is running, closing the main window keeps the timer alive in the tray.
The tray menu provides **Open Calibre 60**, **Start / pause**, and **Quit**.

On GNOME, displaying tray icons may require an AppIndicator/KStatusNotifier extension.

## Keyboard shortcuts

Shortcuts apply when the app has focus and a duration input is not being edited.

| Key | Action |
| --- | --- |
| Space | Start or pause the selected clock |
| L | Record a lap in stopwatch mode |
| R | Reset the selected clock only when stopped |
| Escape | Acknowledge an expired countdown alarm |

## Precision and lifecycle

Timing uses Rust's monotonic `Instant` clock. Elapsed time is calculated from
timestamps rather than by adding a fixed increment on every frame, so delayed
repaints do not accumulate timing drift. Active rendering requests an update about
every 16 ms.

A millisecond display is **not a guarantee of ±1 ms physical accuracy**. OS scheduling,
hardware clocks, input processing, and audio buffering affect observed response times.
The alarm worker uses the countdown's absolute deadline but is not a hard real-time system.

The application does not prevent system sleep or wake a suspended computer. Suspend/resume
behavior of monotonic clocks remains platform-dependent.

Preferences are persisted, but active stopwatch/countdown progress and lap sessions are
not restored after the process exits. Choosing **Quit** from the tray terminates the app.

## Development

| File | Purpose |
| --- | --- |
| `src/main.rs` | Application state, controls, shortcuts, presets, and UI |
| `src/clock.rs` | Timing engine and deterministic unit tests |
| `src/dial.rs` | Vector chronograph drawing |
| `src/alarm.rs` | Independent countdown sound worker |
| `src/notifications.rs` | Native countdown-finished notification |
| `src/tray.rs` | Linux/Windows system tray integration |
| `src/laps.rs` | Lap calculations and CSV export |
| `src/i18n.rs` | French/English translations |
| `src/settings.rs` | Persistent preferences |
| `packaging/linux/` | Debian desktop-entry packaging files |
| `assets/` | Distribution icons and assets |

```bash
cargo test
cargo build --release
cargo fmt
```

Direct dependency versions are pinned in `Cargo.toml`.

## License

See [LICENSE](LICENSE) for the GNU General Public License, version 3.
