package fr.jeromelab.calibre60

import android.content.Context

enum class AppLanguage {
    FRENCH,
    ENGLISH;

    fun pick(fr: String, en: String): String = if (this == FRENCH) fr else en
}

enum class UiThemeMode {
    LIGHT,
    DARK
}

enum class DialStyle {
    CLASSIC,
    NAVY
}

enum class TimerMode {
    STOPWATCH,
    COUNTDOWN
}

class PreferencesStore(context: Context) {
    private val prefs = context.getSharedPreferences("calibre60.android.preferences", Context.MODE_PRIVATE)

    var language: AppLanguage
        get() = enumValueOrDefault(prefs.getString("language", null), AppLanguage.FRENCH)
        set(value) = prefs.edit().putString("language", value.name).apply()

    var uiTheme: UiThemeMode
        get() = enumValueOrDefault(prefs.getString("ui_theme", null), UiThemeMode.DARK)
        set(value) = prefs.edit().putString("ui_theme", value.name).apply()

    var dialStyle: DialStyle
        get() = enumValueOrDefault(prefs.getString("dial_style", null), DialStyle.NAVY)
        set(value) = prefs.edit().putString("dial_style", value.name).apply()

    var sound: Boolean
        get() = prefs.getBoolean("sound", true)
        set(value) = prefs.edit().putBoolean("sound", value).apply()

    private inline fun <reified T : Enum<T>> enumValueOrDefault(raw: String?, fallback: T): T =
        runCatching { enumValueOf<T>(raw ?: "") }.getOrDefault(fallback)
}
