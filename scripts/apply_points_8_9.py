from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"Anchor not found in {path}: {old[:80]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


main = Path("src/main.rs")

replace_once(
    main,
    """    backgrounded: bool,\n    quit_requested: bool,\n}\n""",
    """    backgrounded: bool,\n    quit_requested: bool,\n    compact_mode: bool,\n    normal_window_size: Option<Vec2>,\n}\n""",
)

replace_once(
    main,
    """            backgrounded: false,\n            quit_requested: false,\n        }\n""",
    """            backgrounded: false,\n            quit_requested: false,\n            compact_mode: false,\n            normal_window_size: None,\n        }\n""",
)

compact_methods = r'''    fn set_compact_mode(&mut self, ctx: &egui::Context, compact: bool) {
        if compact == self.compact_mode {
            return;
        }

        if compact {
            self.normal_window_size =
                ctx.input(|input| input.viewport().inner_rect.map(|rect| rect.size()));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(360.0, 90.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(560.0, 110.0)));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(440.0, 620.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                self.normal_window_size
                    .take()
                    .unwrap_or_else(|| Vec2::new(620.0, 900.0)),
            ));
        }

        self.compact_mode = compact;
    }

    fn compact_panel(&mut self, ctx: &egui::Context) {
        let language = self.language;
        let now = Instant::now();
        let displayed = match self.mode {
            Mode::Stopwatch => self.stopwatch.elapsed(now),
            Mode::Countdown => self.countdown.remaining(self.target, now),
        };
        let time_color = if self.mode == Mode::Countdown && self.finished {
            ACCENT
        } else {
            INK
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(if self.mode == Mode::Stopwatch { "⏱" } else { "⏳" })
                        .size(24.0),
                );
                ui.label(
                    RichText::new(format_time(displayed))
                        .monospace()
                        .size(28.0)
                        .color(time_color),
                );

                let action_label = if self.running() {
                    language.tr("pause")
                } else if self.mode == Mode::Countdown
                    && self.countdown.elapsed(Instant::now()) >= self.target
                    && !self.target.is_zero()
                {
                    language.tr("restart")
                } else {
                    language.tr("start_resume")
                };
                let can_start = self.mode == Mode::Stopwatch || !self.target.is_zero();
                if ui
                    .add_enabled(
                        can_start,
                        egui::Button::new(if self.running() { "⏸" } else { "▶" }),
                    )
                    .on_hover_text(action_label)
                    .clicked()
                {
                    self.toggle();
                }

                if self.mode == Mode::Stopwatch
                    && ui
                        .add_enabled(
                            self.stopwatch.running(),
                            egui::Button::new(language.tr("lap")),
                        )
                        .clicked()
                {
                    self.lap();
                }

                if self.finished
                    && ui
                        .button("■")
                        .on_hover_text(language.tr("stop_alarm"))
                        .clicked()
                {
                    self.acknowledge();
                }

                if ui
                    .button(format!("↗ {}", language.tr("normal_view")))
                    .clicked()
                {
                    self.set_compact_mode(ctx, false);
                }
            });
        });
    }

'''

replace_once(
    main,
    """    fn lap_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {\n""",
    compact_methods + "    fn lap_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {\n",
)

replace_once(
    main,
    """        }\n\n        let language = self.language;\n        egui::CentralPanel::default().show(ctx, |ui| {\n""",
    """        }\n\n        if self.compact_mode {\n            self.compact_panel(ctx);\n            if self.stopwatch.running() || self.countdown.running() {\n                ctx.request_repaint_after(Duration::from_millis(16));\n            }\n            return;\n        }\n\n        let language = self.language;\n        egui::CentralPanel::default().show(ctx, |ui| {\n""",
)

replace_once(
    main,
    """                    #[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\n                    if self.tray.is_some() {\n""",
    """                    if ui.button(language.tr(\"compact_mode\")).clicked() {\n                        self.set_compact_mode(ctx, true);\n                    }\n\n                    #[cfg(any(target_os = \"linux\", target_os = \"windows\"))]\n                    if self.tray.is_some() {\n""",
)

# Rust 1.98 warns about ambiguous float fallback in these Stroke constructors.
text = main.read_text(encoding="utf-8")
text = text.replace("Stroke::new(1.0, Color32::WHITE)", "Stroke::new(1.0_f32, Color32::WHITE)")
main.write_text(text, encoding="utf-8")

dial = Path("src/dial.rs")
text = dial.read_text(encoding="utf-8")
text = text.replace("Stroke::new(1.0, MUTED)", "Stroke::new(1.0_f32, MUTED)")
dial.write_text(text, encoding="utf-8")

Path("assets").mkdir(exist_ok=True)
Path("assets/calibre-60.svg").write_text(
    '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="56" fill="#f8f6f0"/>
  <rect x="104" y="26" width="48" height="20" rx="10" fill="#a0342f"/>
  <rect x="118" y="40" width="20" height="24" rx="6" fill="#23272b"/>
  <circle cx="128" cy="140" r="78" fill="none" stroke="#23272b" stroke-width="12"/>
  <circle cx="128" cy="140" r="60" fill="none" stroke="#7d7f7e" stroke-width="3" stroke-dasharray="3 9"/>
  <path d="M128 140 L128 88" stroke="#a0342f" stroke-width="8" stroke-linecap="round"/>
  <path d="M128 140 L166 160" stroke="#23272b" stroke-width="7" stroke-linecap="round"/>
  <circle cx="128" cy="140" r="8" fill="#a0342f"/>
  <text x="128" y="205" text-anchor="middle" font-family="sans-serif" font-size="28" font-weight="700" fill="#23272b">60</text>
</svg>\n''',
    encoding="utf-8",
)

Path("packaging/linux").mkdir(parents=True, exist_ok=True)
Path("packaging/linux/calibre-60.desktop").write_text(
    '''[Desktop Entry]\nType=Application\nName=Calibre 60\nComment=Chronomètre et compte à rebours natif\nComment[en]=Native stopwatch and countdown timer\nExec=calibre-60\nIcon=calibre-60\nTerminal=false\nCategories=Utility;Clock;\nStartupNotify=true\nStartupWMClass=fr.jeromelab.calibre60\n''',
    encoding="utf-8",
)

Path(".github/workflows/release.yml").write_text(
    r'''name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  linux:
    name: Debian 13 package
    runs-on: ubuntu-24.04
    container: debian:13
    timeout-minutes: 35
    env:
      CARGO_TERM_COLOR: always
    steps:
      - name: Install build dependencies
        run: |
          apt-get update
          apt-get install -y --no-install-recommends git curl ca-certificates build-essential pkg-config dpkg-dev libasound2-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev libgl1-mesa-dev
      - uses: actions/checkout@v7
      - name: Install stable Rust
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/calibre-rustup.sh
          sh /tmp/calibre-rustup.sh -y --profile minimal --default-toolchain stable
          echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
      - name: Test and build
        run: |
          cargo test --all-targets
          cargo build --release
      - name: Build Debian package
        shell: bash
        run: |
          set -euo pipefail
          VERSION="${GITHUB_REF_NAME#v}"
          ROOT="dist/calibre-60_${VERSION}_amd64"
          install -Dm755 target/release/calibre-60 "$ROOT/usr/bin/calibre-60"
          install -Dm644 packaging/linux/calibre-60.desktop "$ROOT/usr/share/applications/calibre-60.desktop"
          install -Dm644 assets/calibre-60.svg "$ROOT/usr/share/icons/hicolor/scalable/apps/calibre-60.svg"
          install -Dm644 README.fr.md "$ROOT/usr/share/doc/calibre-60/README.fr.md"
          install -Dm644 LICENSE "$ROOT/usr/share/doc/calibre-60/copyright"
          mkdir -p "$ROOT/DEBIAN"
          cat > "$ROOT/DEBIAN/control" <<EOF
          Package: calibre-60
          Version: ${VERSION}
          Section: utils
          Priority: optional
          Architecture: amd64
          Maintainer: JérômeLab
          Depends: libasound2t64, libxkbcommon0, libwayland-client0, libegl1, libgl1, libx11-6
          Description: Native stopwatch and countdown timer
           Calibre 60 is a native Rust stopwatch and countdown timer with an
           analog chronograph dial, lap timing, configurable alarms and tray mode.
          EOF
          dpkg-deb --root-owner-group --build "$ROOT"
          mv "${ROOT}.deb" "dist/calibre-60_${VERSION}_debian13_amd64.deb"
          tar -C target/release -czf "dist/calibre-60_${VERSION}_linux-x64.tar.gz" calibre-60
      - name: Upload Linux release files
        uses: actions/upload-artifact@v7
        with:
          name: release-linux
          path: |
            dist/*.deb
            dist/*.tar.gz
          if-no-files-found: error

  windows:
    name: Windows x64 package
    runs-on: windows-2022
    timeout-minutes: 35
    env:
      CARGO_TERM_COLOR: always
    steps:
      - uses: actions/checkout@v7
      - name: Install stable Rust
        run: |
          rustup toolchain install stable --profile minimal
          rustup default stable
      - name: Test and build
        run: |
          cargo test --all-targets
          cargo build --release
      - name: Build Windows ZIP
        shell: pwsh
        run: |
          $version = $env:GITHUB_REF_NAME.TrimStart('v')
          New-Item -ItemType Directory -Force -Path dist, release-win | Out-Null
          Copy-Item target/release/calibre-60.exe release-win/
          Copy-Item README.md release-win/
          Copy-Item LICENSE release-win/
          Compress-Archive -Path release-win/* -DestinationPath "dist/calibre-60_${version}_windows-x64.zip"
      - name: Upload Windows release files
        uses: actions/upload-artifact@v7
        with:
          name: release-windows
          path: dist/*.zip
          if-no-files-found: error

  publish:
    name: Publish GitHub Release
    needs: [linux, windows]
    runs-on: ubuntu-24.04
    steps:
      - name: Download release files
        uses: actions/download-artifact@v8
        with:
          pattern: release-*
          path: dist
          merge-multiple: true
      - name: Generate checksums
        run: |
          cd dist
          sha256sum * > SHA256SUMS.txt
      - name: Create GitHub Release
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh release create "$GITHUB_REF_NAME" dist/* \
            --verify-tag \
            --generate-notes \
            --title "Calibre 60 $GITHUB_REF_NAME"
''',
    encoding="utf-8",
)

# The helper and its one-shot workflow remove themselves in the resulting commit.
Path("scripts/apply_points_8_9.py").unlink()
Path(".github/workflows/apply-points-8-9.yml").unlink()
