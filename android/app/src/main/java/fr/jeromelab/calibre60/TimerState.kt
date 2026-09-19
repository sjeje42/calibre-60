package fr.jeromelab.calibre60

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

class MonoClock {
    var accumulatedNanos by mutableLongStateOf(0L)
        private set

    var startedAtNanos by mutableLongStateOf(0L)
        private set

    var running by mutableStateOf(false)
        private set

    fun elapsed(nowNanos: Long): Long {
        val active = if (running) (nowNanos - startedAtNanos).coerceAtLeast(0L) else 0L
        return accumulatedNanos + active
    }

    fun start(nowNanos: Long) {
        if (running) return
        startedAtNanos = nowNanos
        running = true
    }

    fun pause(nowNanos: Long) {
        if (!running) return
        accumulatedNanos = elapsed(nowNanos)
        startedAtNanos = 0L
        running = false
    }

    fun finishAt(valueNanos: Long) {
        accumulatedNanos = valueNanos.coerceAtLeast(0L)
        startedAtNanos = 0L
        running = false
    }

    fun reset() {
        accumulatedNanos = 0L
        startedAtNanos = 0L
        running = false
    }
}
