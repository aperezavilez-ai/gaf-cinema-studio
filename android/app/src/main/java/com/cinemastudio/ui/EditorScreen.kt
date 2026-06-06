package com.cinemastudio.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.Redo
import androidx.compose.material.icons.automirrored.filled.Undo
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EditorScreen(onBack: () -> Unit, vm: EditorViewModel = viewModel()) {
    val context = LocalContext.current
    val scroll = rememberScrollState()

    LaunchedEffect(Unit) {
        vm.onEnterEditor()
    }

    val importLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri ->
        uri?.let { vm.importVideo(context, it) }
    }

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
                .verticalScroll(scroll)
        ) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(16f / 9f)
                    .padding(16.dp)
                    .background(Color(0xFF141414)),
                contentAlignment = Alignment.Center
            ) {
                when {
                    vm.previewBitmap != null -> {
                        Image(
                            bitmap = vm.previewBitmap!!.asImageBitmap(),
                            contentDescription = "Preview",
                            modifier = Modifier.fillMaxSize()
                        )
                    }
                    vm.isDecoding -> CircularProgressIndicator(color = Color.White.copy(0.5f))
                    else -> Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            Icons.Default.VideoLibrary,
                            contentDescription = null,
                            tint = Color.White.copy(alpha = 0.3f),
                            modifier = Modifier.size(48.dp)
                        )
                        Text(
                            if (vm.clipCount == 0) "IMPORT VIDEO TO START" else "PREVIEW",
                            color = Color.White.copy(alpha = 0.25f),
                            letterSpacing = 3.sp,
                            fontSize = 11.sp
                        )
                    }
                }
            }

            if (vm.clipCount == 0) {
                OutlinedButton(
                    onClick = { importLauncher.launch("video/*") },
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                ) {
                    Icon(Icons.Default.VideoLibrary, contentDescription = null, tint = Color.White)
                    Spacer(Modifier.width(8.dp))
                    Text("Import video", color = Color.White)
                }
            } else {
                Row(
                    Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    IconButton(onClick = { vm.splitAtPlayhead() }) {
                        Icon(Icons.Default.ContentCut, "Split", tint = Color.White)
                    }
                    IconButton(onClick = { vm.duplicateAtPlayhead() }) {
                        Icon(Icons.Default.ContentCopy, "Duplicate", tint = Color.White)
                    }
                    IconButton(onClick = { vm.deleteAtPlayhead() }) {
                        Icon(Icons.Default.Delete, "Delete", tint = Color.White)
                    }
                    IconButton(onClick = { vm.undo() }, enabled = vm.canUndo) {
                        Icon(Icons.AutoMirrored.Filled.Undo, "Undo", tint = Color.White.copy(if (vm.canUndo) 1f else 0.3f))
                    }
                    IconButton(onClick = { vm.redo() }, enabled = vm.canRedo) {
                        Icon(Icons.AutoMirrored.Filled.Redo, "Redo", tint = Color.White.copy(if (vm.canRedo) 1f else 0.3f))
                    }
                    IconButton(onClick = { vm.startExport() }, enabled = !vm.isExporting) {
                        Icon(Icons.Default.Share, "Export", tint = Color.White)
                    }
                }
            }

            if (vm.statusMessage.isNotEmpty()) {
                Text(
                    vm.statusMessage,
                    color = Color.White.copy(0.45f),
                    fontSize = 12.sp,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
                )
            }

            if (vm.exportStatus.isNotEmpty()) {
                Text(
                    vm.exportStatus,
                    color = Color.White.copy(0.5f),
                    fontSize = 12.sp,
                    modifier = Modifier.padding(horizontal = 16.dp)
                )
            }

            Row(
                Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(
                    onClick = { if (vm.isPlaying) vm.playbackPause() else vm.playbackPlay() },
                    enabled = vm.clipCount > 0
                ) {
                    Icon(
                        if (vm.isPlaying) Icons.Default.Pause else Icons.Default.PlayArrow,
                        contentDescription = null,
                        tint = Color.White.copy(if (vm.clipCount > 0) 1f else 0.3f)
                    )
                }
                Text(
                    formatTime(vm.playheadMs),
                    color = Color.White.copy(0.6f),
                    fontFamily = FontFamily.Monospace,
                    fontSize = 12.sp
                )
                Slider(
                    value = vm.playheadMs,
                    onValueChange = { vm.scrubTo(it) },
                    valueRange = 0f..vm.durationMs.coerceAtLeast(1f),
                    modifier = Modifier.weight(1f).padding(horizontal = 8.dp),
                    enabled = vm.clipCount > 0,
                    colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Color.White)
                )
                Text(
                    formatTime(vm.durationMs),
                    color = Color.White.copy(0.6f),
                    fontFamily = FontFamily.Monospace,
                    fontSize = 12.sp
                )
            }

            if (vm.clipCount > 0) {
                TimelineStrip(
                    durationMs = vm.durationMs,
                    playheadMs = vm.playheadMs,
                    clips = vm.clips,
                    selectedClipId = vm.selectedClipId,
                    onScrub = { vm.scrubTo(it) },
                    onTrimClip = { id, start, end -> vm.trimClip(id, start, end) },
                    modifier = Modifier.padding(horizontal = 16.dp)
                )

                Text(
                    "Drag white handles to trim · Gold clip = selected",
                    color = Color.White.copy(0.3f),
                    fontSize = 10.sp,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
                )

                TextButton(
                    onClick = { importLauncher.launch("video/*") },
                    modifier = Modifier.align(Alignment.CenterHorizontally)
                ) {
                    Text("+ Add more video", color = Color.White.copy(0.5f), fontSize = 12.sp)
                }
            }

            Spacer(Modifier.height(12.dp))

            CinematicToolsPanel(
                selectedClip = vm.selectedClip,
                activeCameraAngle = vm.activeCameraAngle,
                onLensChange = { vm.applyLens(it) },
                onColorChange = { b, c, s -> vm.applyColor(b, c, s) },
                onCameraAngleChange = { vm.setCameraAngle(it) },
            )

            Spacer(Modifier.height(12.dp))

            GuidancePanel(
                suggestions = vm.suggestions,
                onExecute = { vm.executeSuggestion(it) },
                onDismiss = { vm.dismissSuggestion(it) },
            )

            Spacer(Modifier.height(24.dp))
        }
    }
}

private fun formatTime(ms: Float): String {
    val s = (ms / 1000).toInt()
    return "%d:%02d".format(s / 60, s % 60)
}
