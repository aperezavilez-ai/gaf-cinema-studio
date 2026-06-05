package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EditorScreen(onBack: () -> Unit) {
    var playheadMs by remember { mutableFloatStateOf(0f) }
    var durationMs by remember { mutableFloatStateOf(5000f) }
    var isPlaying by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Editor", color = Color.White) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back", tint = Color.White)
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Black)
            )
        },
        containerColor = Color.Black
    ) { padding ->
        Column(
            Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            // Preview
            Box(
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(16f / 9f)
                    .padding(16.dp)
                    .background(Color(0xFF141414)),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        if (isPlaying) Icons.Default.PlayArrow else Icons.Default.VideoLibrary,
                        contentDescription = null,
                        tint = Color.White.copy(alpha = 0.3f),
                        modifier = Modifier.size(48.dp)
                    )
                    Text(
                        if (isPlaying) "PLAYING" else "PREVIEW",
                        color = Color.White.copy(alpha = 0.25f),
                        letterSpacing = 3.sp,
                        fontSize = 11.sp
                    )
                }
            }

            // Toolbar
            Row(
                Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                IconButton(onClick = {}) { Icon(Icons.Default.ContentCut, "Split", tint = Color.White) }
                IconButton(onClick = {}) { Icon(Icons.Default.Delete, "Delete", tint = Color.White) }
                IconButton(onClick = {}) { Icon(Icons.Default.Undo, "Undo", tint = Color.White.copy(0.3f)) }
                IconButton(onClick = {}) { Icon(Icons.Default.Redo, "Redo", tint = Color.White.copy(0.3f)) }
                IconButton(onClick = {}) { Icon(Icons.Default.Share, "Export", tint = Color.White) }
            }

            // Transport
            Row(
                Modifier.fillMaxWidth().padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = { isPlaying = !isPlaying }) {
                    Icon(
                        if (isPlaying) Icons.Default.Pause else Icons.Default.PlayArrow,
                        contentDescription = null,
                        tint = Color.White
                    )
                }
                Text(formatTime(playheadMs), color = Color.White.copy(0.6f), fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                Slider(
                    value = playheadMs,
                    onValueChange = { playheadMs = it },
                    valueRange = 0f..durationMs,
                    modifier = Modifier.weight(1f).padding(horizontal = 8.dp),
                    colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Color.White)
                )
                Text(formatTime(durationMs), color = Color.White.copy(0.6f), fontFamily = FontFamily.Monospace, fontSize = 12.sp)
            }

            // Timeline strip
            BoxWithConstraints(
                Modifier
                    .fillMaxWidth()
                    .height(80.dp)
                    .padding(horizontal = 16.dp)
                    .background(Color(0xFF1E1E1E))
                    .pointerInput(durationMs) {
                        detectDragGestures { change, _ ->
                            val ratio = (change.position.x / size.width).coerceIn(0f, 1f)
                            playheadMs = ratio * durationMs
                        }
                    }
            ) {
                val trackWidth = maxWidth
                Box(
                    Modifier
                        .fillMaxHeight()
                        .fillMaxWidth(0.6f)
                        .padding(4.dp)
                        .background(Color.White.copy(alpha = 0.15f))
                )
                if (durationMs > 0) {
                    Box(
                        Modifier
                            .fillMaxHeight()
                            .width(2.dp)
                            .offset(x = trackWidth * (playheadMs / durationMs))
                            .background(Color.White)
                    )
                }
            }
        }
    }
}

private fun formatTime(ms: Float): String {
    val s = (ms / 1000).toInt()
    return "%d:%02d".format(s / 60, s % 60)
}
