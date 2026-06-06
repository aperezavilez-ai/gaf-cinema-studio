package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cinemastudio.engine.TimelineClip
import kotlin.math.roundToLong

@Composable
fun TimelineStrip(
    durationMs: Float,
    playheadMs: Float,
    clips: List<TimelineClip>,
    selectedClipId: String?,
    onScrub: (Float) -> Unit,
    onTrimClip: (clipId: String, newStartMs: Long, newEndMs: Long) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (durationMs <= 0f) return

    BoxWithConstraints(
        modifier = modifier
            .fillMaxWidth()
            .height(88.dp)
            .background(Color(0xFF1E1E1E), RoundedCornerShape(4.dp))
            .pointerInput(durationMs, clips.size) {
                detectDragGestures { change, _ ->
                    val ratio = (change.position.x / size.width).coerceIn(0f, 1f)
                    onScrub(ratio * durationMs)
                }
            }
    ) {
        val trackWidth = maxWidth
        val totalPx = constraints.maxWidth.toFloat()

        clips.forEach { clip ->
            val startFrac = clip.startMs / durationMs
            val widthFrac = (clip.durationMs / durationMs).coerceIn(0.02f, 1f)
            val isSelected = clip.id == selectedClipId
            val clipColor = if (isSelected) Color(0xFFC9A227).copy(0.35f) else Color.White.copy(0.15f)

            Box(
                Modifier
                    .align(Alignment.CenterStart)
                    .offset(x = trackWidth * startFrac)
                    .width(trackWidth * widthFrac)
                    .fillMaxHeight()
                    .padding(vertical = 6.dp, horizontal = 1.dp)
                    .background(clipColor, RoundedCornerShape(3.dp))
            ) {
                Text(
                    text = clip.label.take(12),
                    color = Color.White.copy(0.55f),
                    fontSize = 9.sp,
                    modifier = Modifier.align(Alignment.TopStart).padding(4.dp),
                )
                if (clip.fadeInMs > 0 || clip.fadeOutMs > 0) {
                    Text(
                        text = "fade",
                        color = Color(0xFFC9A227).copy(0.8f),
                        fontSize = 8.sp,
                        modifier = Modifier.align(Alignment.BottomEnd).padding(4.dp),
                    )
                }

                if (isSelected) {
                    TrimHandle(
                        modifier = Modifier.align(Alignment.CenterStart),
                        onDrag = { deltaPx, totalPx ->
                            val deltaMs = (deltaPx / totalPx * durationMs).roundToLong()
                            val newStart = (clip.startMs + deltaMs).coerceIn(
                                clip.startMs,
                                clip.startMs + clip.durationMs - 500,
                            )
                            onTrimClip(clip.id, newStart, clip.startMs + clip.durationMs)
                        },
                        totalPx = totalPx,
                    )
                    TrimHandle(
                        modifier = Modifier.align(Alignment.CenterEnd),
                        onDrag = { deltaPx, _ ->
                            val deltaMs = (deltaPx / totalPx * durationMs).roundToLong()
                            val end = clip.startMs + clip.durationMs
                            val newEnd = (end + deltaMs).coerceIn(clip.startMs + 500, end)
                            onTrimClip(clip.id, clip.startMs, newEnd)
                        },
                        totalPx = totalPx,
                    )
                }
            }
        }

        Box(
            Modifier
                .align(Alignment.CenterStart)
                .offset(x = trackWidth * (playheadMs / durationMs))
                .fillMaxHeight()
                .width(2.dp)
                .background(Color.White)
        )
    }
}

@Composable
private fun TrimHandle(
    modifier: Modifier = Modifier,
    totalPx: Float,
    onDrag: (deltaPx: Float, totalPx: Float) -> Unit,
) {
    Box(
        modifier
            .width(10.dp)
            .fillMaxHeight(0.65f)
            .background(Color.White.copy(0.9f), RoundedCornerShape(2.dp))
            .pointerInput(Unit) {
                detectDragGestures { _, dragAmount ->
                    onDrag(dragAmount.x, totalPx)
                }
            }
    )
}
