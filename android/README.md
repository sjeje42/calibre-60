# Calibre 60 — Android

Android companion edition of Calibre 60 v0.2.0.

The Android UI is native Kotlin + Jetpack Compose. The chronograph is drawn entirely in code: no raster dial asset is used.

## Mobile layout

- phone-first responsive layout
- permanent bottom action bar for Start/Pause/Resume, Lap and Reset
- settings behind the gear icon
- persistent French/English language
- persistent light/dark application theme
- persistent classic ivory/navy dial style
- navy dial keeps the red outer rim and the yellow/red 30-minute subdial bezel
- landscape / large-width layout automatically switches to a two-column arrangement
- monotonic timing via `SystemClock.elapsedRealtimeNanos()`

## APK only

This project is intentionally set up for direct APK sideloading. No Google Play publishing configuration is included.

The CI builds a debug-signed APK that can be installed directly on an Android device after allowing installation from the chosen file/browser source.

## Local build

Requirements:

- JDK 17
- Android SDK platform 37
- Android SDK Build Tools 36.0.0
- Gradle 9.6.0

From this directory:

```bash
gradle :app:assembleDebug
```

APK:

```text
app/build/outputs/apk/debug/app-debug.apk
```

Install with ADB:

```bash
adb install -r app/build/outputs/apk/debug/app-debug.apk
```
