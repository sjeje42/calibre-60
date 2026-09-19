package fr.jeromelab.calibre60

import android.app.Activity
import android.media.AudioManager
import android.media.ToneGenerator
import android.os.SystemClock
import android.view.WindowManager
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlin.math.max

private val Accent = Color(0xFFB73530)
private val DarkBackground = Color(0xFF0F141C)
private val DarkSurface = Color(0xFF161E29)
private val LightBackground = Color(0xFFF8F6F0)

@Composable
fun Calibre60Root() {
    val context = LocalContext.current
    val store = remember { PreferencesStore(context.applicationContext) }

    var language by remember { mutableStateOf(store.language) }
    var uiTheme by remember { mutableStateOf(store.uiTheme) }
    var dialStyle by remember { mutableStateOf(store.dialStyle) }
    var sound by remember { mutableStateOf(store.sound) }
    var settingsOpen by remember { mutableStateOf(false) }

    CalibreTheme(uiTheme) {
        if (settingsOpen) {
            SettingsScreen(
                language = language,
                uiTheme = uiTheme,
                dialStyle = dialStyle,
                sound = sound,
                onBack = { settingsOpen = false },
                onLanguage = {
                    language = it
                    store.language = it
                },
                onUiTheme = {
                    uiTheme = it
                    store.uiTheme = it
                },
                onDialStyle = {
                    dialStyle = it
                    store.dialStyle = it
                },
                onSound = {
                    sound = it
                    store.sound = it
                },
            )
        } else {
            MainTimerScreen(
                language = language,
                dialStyle = dialStyle,
                sound = sound,
                onSettings = { settingsOpen = true },
            )
        }
    }
}

@Composable
private fun CalibreTheme(mode: UiThemeMode, content: @Composable () -> Unit) {
    val scheme = if (mode == UiThemeMode.DARK) {
        darkColorScheme(
            primary = Accent,
            secondary = Color(0xFFE8BE30),
            background = DarkBackground,
            surface = DarkSurface,
        )
    } else {
        lightColorScheme(
            primary = Accent,
            secondary = Color(0xFFD2342B),
            background = LightBackground,
            surface = Color(0xFFFFFFFF),
        )
    }

    MaterialTheme(colorScheme = scheme, content = content)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun MainTimerScreen(
    language: AppLanguage,
    dialStyle: DialStyle,
    sound: Boolean,
    onSettings: () -> Unit,
) {
    var mode by remember { mutableStateOf(TimerMode.STOPWATCH) }
    val stopwatch = remember { MonoClock() }
    val countdown = remember { MonoClock() }
    var targetNanos by remember { mutableLongStateOf(5L * 60L * 1_000_000_000L) }
    var nowNanos by remember { mutableLongStateOf(SystemClock.elapsedRealtimeNanos()) }
    var finished by remember { mutableStateOf(false) }
    var showDurationDialog by remember { mutableStateOf(false) }
    val laps = remember { mutableStateListOf<Long>() }

    val tone = remember { ToneGenerator(AudioManager.STREAM_ALARM, 75) }
    DisposableEffect(Unit) {
        onDispose { tone.release() }
    }

    val anyRunning = stopwatch.running || countdown.running
    val activity = LocalContext.current as? Activity
    DisposableEffect(anyRunning) {
        if (anyRunning) {
            activity?.window?.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        } else {
            activity?.window?.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
        onDispose {
            if (!anyRunning) activity?.window?.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
    }

    LaunchedEffect(stopwatch.running, countdown.running, sound) {
        while (stopwatch.running || countdown.running) {
            val now = SystemClock.elapsedRealtimeNanos()
            nowNanos = now

            if (countdown.running && countdown.elapsed(now) >= targetNanos) {
                countdown.finishAt(targetNanos)
                finished = true
                if (sound) tone.startTone(ToneGenerator.TONE_PROP_BEEP2, 700)
            }
            delay(16L)
        }
        nowNanos = SystemClock.elapsedRealtimeNanos()
    }

    fun selectedClock(): MonoClock = if (mode == TimerMode.STOPWATCH) stopwatch else countdown
    fun displayedNanos(): Long {
        val elapsed = selectedClock().elapsed(nowNanos)
        return if (mode == TimerMode.COUNTDOWN) (targetNanos - elapsed).coerceAtLeast(0L) else elapsed
    }

    fun toggle() {
        val now = SystemClock.elapsedRealtimeNanos()
        nowNanos = now
        val clock = selectedClock()

        if (clock.running) {
            clock.pause(now)
            return
        }

        if (mode == TimerMode.COUNTDOWN && targetNanos <= 0L) return
        if (mode == TimerMode.COUNTDOWN && clock.elapsed(now) >= targetNanos) {
            clock.reset()
        }
        finished = false
        clock.start(now)
    }

    fun reset() {
        val clock = selectedClock()
        if (clock.running) return
        clock.reset()
        finished = false
        if (mode == TimerMode.STOPWATCH) laps.clear()
        nowNanos = SystemClock.elapsedRealtimeNanos()
    }

    fun lap() {
        if (mode != TimerMode.STOPWATCH || !stopwatch.running) return
        val now = SystemClock.elapsedRealtimeNanos()
        nowNanos = now
        laps.add(stopwatch.elapsed(now))
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "CALIBRE 60",
                        fontWeight = FontWeight.SemiBold,
                        letterSpacing = 1.5.sp,
                    )
                },
                actions = {
                    IconButton(onClick = onSettings) {
                        Icon(
                            imageVector = Icons.Filled.Settings,
                            contentDescription = language.pick("Réglages", "Settings"),
                        )
                    }
                },
            )
        },
        bottomBar = {
            BottomActionBar(
                language = language,
                mode = mode,
                clock = selectedClock(),
                elapsedNanos = selectedClock().elapsed(nowNanos),
                onToggle = ::toggle,
                onLap = ::lap,
                onReset = ::reset,
            )
        },
    ) { innerPadding ->
        MainResponsiveContent(
            modifier = Modifier.padding(innerPadding),
            language = language,
            mode = mode,
            onMode = {
                mode = it
                finished = false
                nowNanos = SystemClock.elapsedRealtimeNanos()
            },
            displayedNanos = displayedNanos(),
            dialDurationNanos = selectedClock().elapsed(nowNanos),
            dialStyle = dialStyle,
            running = selectedClock().running,
            finished = finished,
            laps = laps,
            countdownTargetNanos = targetNanos,
            countdownRunning = countdown.running,
            onPreset = { minutes ->
                if (!countdown.running) {
                    targetNanos = minutes * 60L * 1_000_000_000L
                    countdown.reset()
                    finished = false
                    nowNanos = SystemClock.elapsedRealtimeNanos()
                }
            },
            onCustomDuration = { showDurationDialog = true },
        )
    }

    if (showDurationDialog) {
        DurationDialog(
            language = language,
            initialSeconds = targetNanos / 1_000_000_000L,
            onDismiss = { showDurationDialog = false },
            onConfirm = { seconds ->
                targetNanos = seconds.coerceIn(0L, 359_999L) * 1_000_000_000L
                countdown.reset()
                finished = false
                showDurationDialog = false
                nowNanos = SystemClock.elapsedRealtimeNanos()
            },
        )
    }
}

@Composable
private fun MainResponsiveContent(
    modifier: Modifier,
    language: AppLanguage,
    mode: TimerMode,
    onMode: (TimerMode) -> Unit,
    displayedNanos: Long,
    dialDurationNanos: Long,
    dialStyle: DialStyle,
    running: Boolean,
    finished: Boolean,
    laps: List<Long>,
    countdownTargetNanos: Long,
    countdownRunning: Boolean,
    onPreset: (Long) -> Unit,
    onCustomDuration: () -> Unit,
) {
    BoxWithConstraints(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 14.dp, vertical = 8.dp),
    ) {
        val wide = maxWidth >= 700.dp

        if (wide) {
            Row(
                modifier = Modifier.fillMaxSize(),
                horizontalArrangement = Arrangement.spacedBy(20.dp),
            ) {
                Column(
                    modifier = Modifier
                        .weight(1.15f)
                        .verticalScroll(rememberScrollState()),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    DigitalReadout(language, displayedNanos, running, finished)
                    ChronographDial(
                        modifier = Modifier
                            .fillMaxWidth()
                            .aspectRatio(1f),
                        durationNanos = dialDurationNanos,
                        mode = mode,
                        style = dialStyle,
                    )
                }

                Column(
                    modifier = Modifier
                        .weight(0.85f)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                ) {
                    ModeSelector(language, mode, onMode)
                    if (mode == TimerMode.COUNTDOWN) {
                        CountdownControls(
                            language = language,
                            targetNanos = countdownTargetNanos,
                            running = countdownRunning,
                            onPreset = onPreset,
                            onCustomDuration = onCustomDuration,
                        )
                    } else {
                        LapPanel(language, laps)
                    }
                }
            }
        } else {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState()),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                ModeSelector(language, mode, onMode)
                DigitalReadout(language, displayedNanos, running, finished)
                ChronographDial(
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(1f),
                    durationNanos = dialDurationNanos,
                    mode = mode,
                    style = dialStyle,
                )

                if (mode == TimerMode.COUNTDOWN) {
                    CountdownControls(
                        language = language,
                        targetNanos = countdownTargetNanos,
                        running = countdownRunning,
                        onPreset = onPreset,
                        onCustomDuration = onCustomDuration,
                    )
                } else {
                    LapPanel(language, laps)
                }

                Spacer(Modifier.height(8.dp))
            }
        }
    }
}

@Composable
private fun ModeSelector(
    language: AppLanguage,
    mode: TimerMode,
    onMode: (TimerMode) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        val stopwatchSelected = mode == TimerMode.STOPWATCH
        if (stopwatchSelected) {
            Button(onClick = { onMode(TimerMode.STOPWATCH) }, modifier = Modifier.weight(1f)) {
                Text(language.pick("Chronomètre", "Stopwatch"))
            }
        } else {
            OutlinedButton(onClick = { onMode(TimerMode.STOPWATCH) }, modifier = Modifier.weight(1f)) {
                Text(language.pick("Chronomètre", "Stopwatch"))
            }
        }

        if (!stopwatchSelected) {
            Button(onClick = { onMode(TimerMode.COUNTDOWN) }, modifier = Modifier.weight(1f)) {
                Text(language.pick("Compte à rebours", "Countdown"))
            }
        } else {
            OutlinedButton(onClick = { onMode(TimerMode.COUNTDOWN) }, modifier = Modifier.weight(1f)) {
                Text(language.pick("Compte à rebours", "Countdown"))
            }
        }
    }
}

@Composable
private fun DigitalReadout(
    language: AppLanguage,
    nanos: Long,
    running: Boolean,
    finished: Boolean,
) {
    Text(
        text = formatNanos(nanos),
        fontFamily = FontFamily.Monospace,
        fontWeight = FontWeight.Medium,
        fontSize = 42.sp,
        maxLines = 1,
    )
    Text(
        text = when {
            finished -> language.pick("TERMINÉ", "FINISHED")
            running -> language.pick("EN COURS", "RUNNING")
            else -> language.pick("À L'ARRÊT", "STOPPED")
        },
        style = MaterialTheme.typography.labelMedium,
        color = if (finished) Accent else MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun CountdownControls(
    language: AppLanguage,
    targetNanos: Long,
    running: Boolean,
    onPreset: (Long) -> Unit,
    onCustomDuration: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                language.pick("Durée du compte à rebours", "Countdown duration"),
                fontWeight = FontWeight.SemiBold,
            )
            Text(
                formatNanos(targetNanos).substringBefore('.'),
                fontFamily = FontFamily.Monospace,
                fontSize = 24.sp,
            )

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                listOf(1L, 3L, 5L, 10L, 25L).forEach { minutes ->
                    FilterChip(
                        selected = targetNanos == minutes * 60L * 1_000_000_000L,
                        onClick = { onPreset(minutes) },
                        enabled = !running,
                        label = { Text("$minutes min") },
                    )
                }
            }

            OutlinedButton(
                onClick = onCustomDuration,
                enabled = !running,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(language.pick("Régler une durée…", "Set duration…"))
            }
        }
    }
}

@Composable
private fun LapPanel(language: AppLanguage, laps: List<Long>) {
    if (laps.isEmpty()) return

    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(
                language.pick("Tours", "Laps"),
                fontWeight = FontWeight.SemiBold,
            )
            laps.asReversed().take(8).forEachIndexed { reverseIndex, value ->
                val lapNumber = laps.size - reverseIndex
                Row(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        language.pick("Tour", "Lap") + " " + lapNumber,
                        modifier = Modifier.weight(1f),
                    )
                    Text(
                        formatNanos(value),
                        fontFamily = FontFamily.Monospace,
                    )
                }
            }
        }
    }
}

@Composable
private fun BottomActionBar(
    language: AppLanguage,
    mode: TimerMode,
    clock: MonoClock,
    elapsedNanos: Long,
    onToggle: () -> Unit,
    onLap: () -> Unit,
    onReset: () -> Unit,
) {
    Surface(tonalElevation = 4.dp) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(horizontal = 10.dp, top = 10.dp, bottom = 14.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            val startText = when {
                clock.running -> language.pick("Pause", "Pause")
                elapsedNanos > 0L -> language.pick("Reprendre", "Resume")
                else -> language.pick("Démarrer", "Start")
            }

            Button(
                onClick = onToggle,
                modifier = Modifier.weight(1.25f),
                colors = ButtonDefaults.buttonColors(containerColor = Accent),
                contentPadding = PaddingValues(horizontal = 6.dp, vertical = 12.dp),
            ) {
                Text(startText, maxLines = 1)
            }

            OutlinedButton(
                onClick = onLap,
                enabled = mode == TimerMode.STOPWATCH && clock.running,
                modifier = Modifier.weight(0.8f),
                contentPadding = PaddingValues(horizontal = 4.dp, vertical = 12.dp),
            ) {
                Text(language.pick("Tour", "Lap"), maxLines = 1)
            }

            OutlinedButton(
                onClick = onReset,
                enabled = !clock.running && elapsedNanos > 0L,
                modifier = Modifier.weight(0.9f),
                contentPadding = PaddingValues(horizontal = 4.dp, vertical = 12.dp),
            ) {
                Text(language.pick("Réinit.", "Reset"), maxLines = 1)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SettingsScreen(
    language: AppLanguage,
    uiTheme: UiThemeMode,
    dialStyle: DialStyle,
    sound: Boolean,
    onBack: () -> Unit,
    onLanguage: (AppLanguage) -> Unit,
    onUiTheme: (UiThemeMode) -> Unit,
    onDialStyle: (DialStyle) -> Unit,
    onSound: (Boolean) -> Unit,
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(language.pick("Réglages", "Settings")) },
                navigationIcon = {
                    TextButton(onClick = onBack) {
                        Text("‹", fontSize = 32.sp)
                    }
                },
            )
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(rememberScrollState())
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(18.dp),
        ) {
            SettingsSectionTitle(language.pick("Apparence", "Appearance"))
            ChoiceRow(
                firstLabel = language.pick("Clair", "Light"),
                secondLabel = language.pick("Sombre", "Dark"),
                firstSelected = uiTheme == UiThemeMode.LIGHT,
                onFirst = { onUiTheme(UiThemeMode.LIGHT) },
                onSecond = { onUiTheme(UiThemeMode.DARK) },
            )

            SettingsSectionTitle(language.pick("Cadran", "Dial"))
            ChoiceRow(
                firstLabel = language.pick("Ivoire classique", "Classic ivory"),
                secondLabel = language.pick("Bleu marine", "Navy blue"),
                firstSelected = dialStyle == DialStyle.CLASSIC,
                onFirst = { onDialStyle(DialStyle.CLASSIC) },
                onSecond = { onDialStyle(DialStyle.NAVY) },
            )

            HorizontalDivider()

            SettingsSectionTitle(language.pick("Langue", "Language"))
            ChoiceRow(
                firstLabel = "Français",
                secondLabel = "English",
                firstSelected = language == AppLanguage.FRENCH,
                onFirst = { onLanguage(AppLanguage.FRENCH) },
                onSecond = { onLanguage(AppLanguage.ENGLISH) },
            )

            HorizontalDivider()

            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(language.pick("Son", "Sound"), fontWeight = FontWeight.SemiBold)
                    Text(
                        language.pick(
                            "Signal sonore à la fin du compte à rebours",
                            "Audible signal when the countdown finishes",
                        ),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Spacer(Modifier.width(12.dp))
                Switch(checked = sound, onCheckedChange = onSound)
            }

            HorizontalDivider()
            Text(
                "Calibre 60 Android v0.2.0",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                language.pick(
                    "APK autonome — aucune publication Play Store requise.",
                    "Standalone APK — no Play Store publishing required.",
                ),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun SettingsSectionTitle(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.titleMedium,
        fontWeight = FontWeight.SemiBold,
    )
}

@Composable
private fun ChoiceRow(
    firstLabel: String,
    secondLabel: String,
    firstSelected: Boolean,
    onFirst: () -> Unit,
    onSecond: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        FilterChip(
            selected = firstSelected,
            onClick = onFirst,
            label = { Text(firstLabel) },
            modifier = Modifier.weight(1f),
        )
        FilterChip(
            selected = !firstSelected,
            onClick = onSecond,
            label = { Text(secondLabel) },
            modifier = Modifier.weight(1f),
        )
    }
}

@Composable
private fun DurationDialog(
    language: AppLanguage,
    initialSeconds: Long,
    onDismiss: () -> Unit,
    onConfirm: (Long) -> Unit,
) {
    var hours by remember { mutableStateOf((initialSeconds / 3600L).toString()) }
    var minutes by remember { mutableStateOf(((initialSeconds / 60L) % 60L).toString()) }
    var seconds by remember { mutableStateOf((initialSeconds % 60L).toString()) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(language.pick("Régler la durée", "Set duration")) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedTextField(
                    value = hours,
                    onValueChange = { hours = it.filter(Char::isDigit).take(2) },
                    label = { Text(language.pick("Heures", "Hours")) },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    singleLine = true,
                )
                OutlinedTextField(
                    value = minutes,
                    onValueChange = { minutes = it.filter(Char::isDigit).take(2) },
                    label = { Text(language.pick("Minutes", "Minutes")) },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    singleLine = true,
                )
                OutlinedTextField(
                    value = seconds,
                    onValueChange = { seconds = it.filter(Char::isDigit).take(2) },
                    label = { Text(language.pick("Secondes", "Seconds")) },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    singleLine = true,
                )
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    val h = (hours.toLongOrNull() ?: 0L).coerceIn(0L, 99L)
                    val m = (minutes.toLongOrNull() ?: 0L).coerceIn(0L, 59L)
                    val s = (seconds.toLongOrNull() ?: 0L).coerceIn(0L, 59L)
                    onConfirm(h * 3600L + m * 60L + s)
                },
            ) {
                Text(language.pick("Appliquer", "Apply"))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(language.pick("Annuler", "Cancel"))
            }
        },
    )
}

private fun formatNanos(value: Long): String {
    val safe = max(0L, value)
    val totalMillis = safe / 1_000_000L
    val hours = totalMillis / 3_600_000L
    val minutes = (totalMillis / 60_000L) % 60L
    val seconds = (totalMillis / 1_000L) % 60L
    val millis = totalMillis % 1_000L
    return "%02d:%02d:%02d.%03d".format(hours, minutes, seconds, millis)
}
