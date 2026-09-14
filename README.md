# Calibre 60

**A native Rust stopwatch and countdown timer with a classic analog chronograph dial.**

English · [Français](README.fr.md)

Calibre 60 combines an ivory dial, a sweeping seconds hand, a 30-minute subdial,
and a digital hours/minutes/seconds/milliseconds display. Built for Linux
(including Debian 13) and Windows with egui/eframe and rodio.

The application interface is currently **in French**. This README is the English
user guide; an English interface is not implemented in this version.

## Features

- Independent stopwatch and countdown: switching tabs does not stop either clock.
- Start, pause, resume, and reset.
- Lap duration and cumulative time, newest lap first.
- Copy all recorded laps to the clipboard as semicolon-separated CSV.
- Countdown settings up to 99 h 59 min 59 s and presets for 1, 3, 5, 10, or 25 minutes.
- Repeating audible alarm, sound toggle, and a visible expiry message in either tab.
- A separate alarm thread, independent of window repainting.
- Resizable, high-DPI-aware vector interface with keyboard shortcuts.
- No account, web service, or image asset required at runtime.

## Status and downloads

This is an initial source release. The [CI workflow](../../actions/workflows/ci.yml)
builds and runs the timing unit tests on Debian 13 and Windows.
Check the latest run for the actual validation result: the presence of a workflow
does not mean the build has passed.

After a successful run, download the corresponding artifact from its **Artifacts**
section: `calibre-60-debian13-x64` or `calibre-60-windows-x64`.
Access requires permission to this private repository.
There is currently no installer or signed release package.

CI checks compilation and timing calculations. It does not validate the actual
window, display scaling, clipboard, audio output, or device suspend behavior.
These need testing on a desktop with a display and sound device.

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

Ensure `~/.local/bin` is on your PATH, then run `calibre-60` from a terminal.

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
CI builds with the MSVC toolchain on Windows Server 2022; actual desktop behavior
still needs checking on your Windows installation.

## Using Calibre 60

| French control | Meaning |
| --- | --- |
| Chronomètre | Stopwatch tab |
| Compte à rebours | Countdown tab |
| Démarrer / Reprendre | Start or resume the selected clock |
| Pause | Pause the selected clock |
| Tour | Record a lap while the stopwatch is running |
| Réinitialiser | Reset the selected clock while stopped; also clears its laps |
| Appliquer | Apply the edited duration and reset the countdown |
| Relancer | Start the completed countdown again with the applied duration |
| Copier CSV | Copy the recorded laps to the clipboard |
| Son | Enable or mute the countdown sound |
| Arrêter l’alarme | Acknowledge the countdown alarm |

For a custom countdown, edit the duration **while stopped**, then click
**Appliquer**. Editing the fields alone does not change the active duration.
Applying a duration or choosing a preset discards the paused countdown progress.
A zero-duration countdown cannot start.

The main hand completes one revolution every 60 seconds; the small hand completes
one every 30 minutes. Both show the remaining duration in countdown mode and move
backwards as that duration decreases. Read the digital display for the complete
duration, including hours.

Click **Copier CSV** and paste into a spreadsheet or text editor. The three
columns are lap number, lap duration, and cumulative stopwatch time.
CSV headers remain in French.

## Keyboard shortcuts

Shortcuts apply when the app has focus and a duration input is not being edited.

| Key | Action |
| --- | --- |
| Space | Start or pause the selected clock |
| L | Record a lap in the stopwatch tab |
| R | Reset the selected clock only when it is stopped |
| Escape | Acknowledge an expired countdown alarm |

## Precision and lifecycle

Timing uses Rust's [monotonic Instant clock](https://doc.rust-lang.org/std/time/struct.Instant.html),
with integer durations. Elapsed time is calculated from timestamps rather than
adding a fixed increment on each frame. A delayed repaint does not accumulate
timing drift. Active rendering requests an update approximately every 16 ms;
not every millisecond is rendered.

A millisecond display is **not a guarantee of ±1 ms physical accuracy**.
Input processing, OS scheduling, the hardware clock, and audio buffering affect
observed response times. The alarm thread uses the countdown's absolute deadline,
but cannot provide hard real-time audio delivery.

Keep the computer awake for continuous timing. Suspend/resume treatment of
`Instant` is platform-dependent; the app does not prevent sleep, wake the machine,
or guarantee an alarm while it is suspended.

Closing the app stops the alarm and discards timers, laps, and settings.
There is no background service, system tray, or session persistence.
If audio initialization fails, an error appears in the app and the visual alert
remains available. Restart the app after fixing the audio output.

## Development

| File | Purpose |
| --- | --- |
| `src/main.rs` | Application state, controls, shortcuts, and lap display |
| `src/clock.rs` | Timing engine and deterministic unit tests |
| `src/dial.rs` | Vector chronograph drawing |
| `src/alarm.rs` | Independent countdown sound worker |

```bash
cargo test
cargo build --release
cargo fmt
```

Direct dependency versions are pinned in `Cargo.toml`.
Cargo generates `Cargo.lock` on the first build; it is intentionally not ignored.
This initial source commit does not contain a generated lockfile.
The Debian CI job uploads its generated lockfile alongside the executable.
Commit a generated and validated lockfile to fix transitive dependency versions
for future builds; the current CI resolves them independently on each platform.

## License

See [LICENSE](LICENSE) for the existing GNU General Public License, version 3.
