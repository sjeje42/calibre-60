package fr.jeromelab.calibre60

import android.graphics.Paint
import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

private val Paper = Color(0xFFF8F6F0)
private val Ink = Color(0xFF23272B)
private val Muted = Color(0xFF7D7F7E)
private val Accent = Color(0xFFA0342F)

private val Navy = Color(0xFF122C4E)
private val NavySubdial = Color(0xFF16355B)
private val NavyBezel = Color(0xFF0E1622)
private val NavyMuted = Color(0xFFAABED3)
private val NavyFineTick = Color(0xFF6A8BAC)
private val RacingYellow = Color(0xFFE8BE30)
private val RacingRed = Color(0xFFD2342B)

private data class DialPalette(
    val face: Color,
    val bezel: Color,
    val text: Color,
    val muted: Color,
    val fineTick: Color,
    val subFace: Color,
    val subText: Color,
    val bezelTick: Color,
    val stopwatchHand: Color,
    val countdownHand: Color,
)

private fun palette(style: DialStyle): DialPalette =
    when (style) {
        DialStyle.CLASSIC -> DialPalette(
            face = Paper,
            bezel = Ink,
            text = Ink,
            muted = Muted,
            fineTick = Color(0xFFB4B1AA),
            subFace = Color(0xFFEEEBE2),
            subText = Muted,
            bezelTick = Paper,
            stopwatchHand = Ink,
            countdownHand = Accent,
        )

        DialStyle.NAVY -> DialPalette(
            face = Navy,
            bezel = NavyBezel,
            text = Paper,
            muted = NavyMuted,
            fineTick = NavyFineTick,
            subFace = NavySubdial,
            subText = Paper,
            bezelTick = Paper,
            stopwatchHand = Paper,
            countdownHand = RacingRed,
        )
    }

private fun radial(center: Offset, radius: Float, turns: Double): Offset {
    val normalized = ((turns % 1.0) + 1.0) % 1.0
    val angle = normalized * (2.0 * PI) - (PI / 2.0)
    return Offset(
        x = center.x + cos(angle).toFloat() * radius,
        y = center.y + sin(angle).toFloat() * radius,
    )
}

private fun DrawScope.centeredText(text: String, point: Offset, textSize: Float, color: Color) {
    val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        this.color = color.toArgb()
        this.textSize = textSize
        textAlign = Paint.Align.CENTER
    }
    val metrics = paint.fontMetrics
    val baseline = point.y - (metrics.ascent + metrics.descent) / 2f
    drawContext.canvas.nativeCanvas.drawText(text, point.x, baseline, paint)
}

@Composable
fun ChronographDial(
    modifier: Modifier = Modifier,
    durationNanos: Long,
    mode: TimerMode,
    style: DialStyle,
) {
    Canvas(modifier = modifier) {
        val dimension = size.minDimension
        val center = Offset(size.width / 2f, size.height / 2f)
        val radius = dimension * 0.405f
        val scale = dimension / 520f
        val seconds = durationNanos.coerceAtLeast(0L) / 1_000_000_000.0
        val colors = palette(style)

        drawCircle(
            color = Color.Black.copy(alpha = if (style == DialStyle.NAVY) 0.20f else 0.10f),
            radius = radius * 1.075f,
            center = center + Offset(0f, 5f * scale),
        )
        drawCircle(colors.bezel, radius = radius * 1.07f, center = center)

        if (style == DialStyle.NAVY) {
            drawCircle(
                color = RacingRed,
                radius = radius * 1.072f,
                center = center,
                style = androidx.compose.ui.graphics.drawscope.Stroke(width = 4f * scale),
            )
        }

        drawCircle(colors.face, radius = radius * 1.025f, center = center)
        drawCircle(
            color = colors.muted,
            radius = radius * 0.985f,
            center = center,
            style = androidx.compose.ui.graphics.drawscope.Stroke(width = 1f * scale),
        )

        for (index in 0 until 120) {
            if (index % 2 == 0) {
                drawLine(
                    color = colors.bezelTick,
                    start = radial(center, radius * 1.035f, index / 120.0),
                    end = radial(center, radius * 1.06f, index / 120.0),
                    strokeWidth = 5f * scale,
                    cap = StrokeCap.Square,
                )
            }
        }

        for (index in 0 until 300) {
            val major = index % 25 == 0
            val second = index % 5 == 0
            val inner = when {
                major -> 0.84f
                second -> 0.885f
                else -> 0.925f
            }
            drawLine(
                color = if (second) colors.text else colors.fineTick,
                start = radial(center, radius * inner, index / 300.0),
                end = radial(center, radius * 0.97f, index / 300.0),
                strokeWidth = (if (major) 2f else 0.8f) * scale,
            )
        }

        for (index in 0 until 12) {
            val value = if (index == 0) 60 else index * 5
            centeredText(
                text = value.toString(),
                point = radial(center, radius * 1.165f, index / 12.0),
                textSize = 23f * scale,
                color = colors.text,
            )
        }

        val subCenter = center - Offset(0f, radius * 0.44f)
        val subRadius = radius * 0.245f
        drawCircle(colors.subFace, radius = subRadius, center = subCenter)

        if (style == DialStyle.NAVY) {
            drawCircle(
                color = NavyBezel,
                radius = subRadius * 1.16f,
                center = subCenter,
                style = androidx.compose.ui.graphics.drawscope.Stroke(width = 1.2f * scale),
            )
            for (index in 0 until 60) {
                val color = if (index < 45) RacingYellow else RacingRed
                drawLine(
                    color = color,
                    start = radial(subCenter, subRadius * 1.025f, index / 60.0),
                    end = radial(subCenter, subRadius * 1.14f, index / 60.0),
                    strokeWidth = 3.2f * scale,
                    cap = StrokeCap.Square,
                )
            }
        } else {
            drawCircle(
                color = colors.muted,
                radius = subRadius,
                center = subCenter,
                style = androidx.compose.ui.graphics.drawscope.Stroke(width = 1f * scale),
            )
        }

        for (index in 0 until 30) {
            drawLine(
                color = colors.muted,
                start = radial(
                    subCenter,
                    subRadius * if (index % 5 == 0) 0.77f else 0.88f,
                    index / 30.0,
                ),
                end = radial(subCenter, subRadius * 0.98f, index / 30.0),
                strokeWidth = 0.9f * scale,
            )
        }

        for (index in 0 until 6) {
            val value = if (index == 0) 30 else index * 5
            centeredText(
                text = value.toString(),
                point = radial(subCenter, subRadius * 0.62f, index / 6.0),
                textSize = 10f * scale,
                color = colors.subText,
            )
        }

        val subTurns = (seconds % 1800.0) / 1800.0
        drawLine(
            color = colors.text,
            start = radial(subCenter, subRadius * 0.12f, subTurns + 0.5),
            end = radial(subCenter, subRadius * 0.77f, subTurns),
            strokeWidth = 2f * scale,
        )
        drawCircle(colors.text, radius = 3f * scale, center = subCenter)

        centeredText("CALIBRE 60", center + Offset(0f, radius * 0.39f), 23f * scale, colors.text)
        centeredText("JÉRÔMELAB", center + Offset(0f, radius * 0.58f), 11f * scale, colors.muted)
        centeredText(
            "60 SECONDES / 30 MINUTES",
            center + Offset(0f, radius * 0.69f),
            8f * scale,
            colors.muted,
        )

        val handColor = if (mode == TimerMode.STOPWATCH) colors.stopwatchHand else colors.countdownHand
        val turns = (seconds % 60.0) / 60.0
        drawLine(
            color = handColor,
            start = radial(center, radius * 0.145f, turns + 0.5),
            end = radial(center, radius * 0.91f, turns),
            strokeWidth = 3.5f * scale,
        )
        drawCircle(handColor, radius = 8f * scale, center = center)
        drawCircle(colors.face, radius = 2.4f * scale, center = center)
    }
}
