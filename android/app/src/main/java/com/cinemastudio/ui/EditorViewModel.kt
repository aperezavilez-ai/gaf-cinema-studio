package com.cinemastudio.ui

import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.cinemastudio.engine.AiSuggestionItem
import com.cinemastudio.engine.ColorGradeUtil
import com.cinemastudio.engine.EngineBridge
import com.cinemastudio.engine.MediaHelper
import com.cinemastudio.engine.ProjectSession
import com.cinemastudio.engine.TimelineClip
import com.cinemastudio.engine.VideoDecoderService
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class EditorViewModel : ViewModel() {
    var playheadMs by mutableStateOf(0f)
    var durationMs by mutableStateOf(1000f)
    var isPlaying by mutableStateOf(false)
    var previewBitmap by mutableStateOf<android.graphics.Bitmap?>(null)
    var isDecoding by mutableStateOf(false)
    var exportStatus by mutableStateOf("")
    var previewPath: String? = null
    var clipCount by mutableStateOf(0)
    var clips by mutableStateOf<List<TimelineClip>>(emptyList())
    var selectedClipId by mutableStateOf<String?>(null)
    var activeCameraAngle by mutableIntStateOf(1)
    var canUndo by mutableStateOf(false)
    var canRedo by mutableStateOf(false)
    var statusMessage by mutableStateOf("")
    var isExporting by mutableStateOf(false)
    var suggestions by mutableStateOf<List<AiSuggestionItem>>(emptyList())

    val selectedClip: TimelineClip?
        get() = clips.find { it.id == selectedClipId }

    private var playbackJob: Job? = null
    private var exportJob: Job? = null
    private val decoder = VideoDecoderService()
    private var decodeTimeMs: Long = 0
    private var gradeBrightness = 0f
    private var gradeContrast = 1f
    private var gradeSaturation = 1f

    fun onEnterEditor() {
        if (!ProjectSession.hasProject()) {
            statusMessage = "Create a project from Home first"
            return
        }
        syncTimeline()
        loadSuggestions()
    }

    fun importVideo(context: Context, uri: Uri) {
        viewModelScope.launch {
            statusMessage = "Importing…"
            runCatching {
                val cachePath = MediaHelper.copyUriToCache(context, uri)
                EngineBridge.importAndAddClip(cachePath)
                syncTimeline()
                scrubTo(0f)
                loadSuggestions()
                statusMessage = "Video imported"
            }.onFailure {
                statusMessage = "Import failed: ${it.message}"
            }
        }
    }

    fun scrubTo(ms: Float) {
        val frame = EngineBridge.scrubTo(ms.toLong())
        playheadMs = frame.timeMs.toFloat()
        frame.primaryPath?.let { previewPath = it }
        decodeTimeMs = frame.sourceTimeMs ?: frame.timeMs
        updateSelectedClip()
        updateGradeFromSelected()
        refreshPreview()
    }

    fun playbackPlay() {
        if (durationMs <= 0f || clipCount == 0) return
        isPlaying = true
        playbackJob?.cancel()
        playbackJob = viewModelScope.launch {
            while (isActive && isPlaying) {
                delay(42)
                val next = (playheadMs + 42f).coerceAtMost(durationMs)
                scrubTo(next)
                if (next >= durationMs) playbackPause()
            }
        }
    }

    fun playbackPause() {
        isPlaying = false
        playbackJob?.cancel()
    }

    fun splitAtPlayhead() {
        if (EngineBridge.splitAtPlayhead()) {
            syncTimeline()
            loadSuggestions()
            statusMessage = "Clip split"
        } else {
            statusMessage = "Move playhead inside a clip to split"
        }
    }

    fun duplicateAtPlayhead() {
        if (EngineBridge.duplicateAtPlayhead()) {
            syncTimeline()
            loadSuggestions()
            statusMessage = "Toma duplicada"
        } else {
            statusMessage = "No clip at playhead to duplicate"
        }
    }

    fun deleteAtPlayhead() {
        if (EngineBridge.deleteAtPlayhead()) {
            syncTimeline()
            loadSuggestions()
            statusMessage = "Clip deleted"
        } else {
            statusMessage = "No clip at playhead"
        }
    }

    fun trimClip(clipId: String, newStartMs: Long, newEndMs: Long) {
        if (EngineBridge.trimClip(clipId, newStartMs, newEndMs)) {
            syncTimeline()
            loadSuggestions()
            statusMessage = "Clip trimmed"
        }
    }

    fun applyLens(preset: String) {
        val clip = selectedClip ?: return
        if (EngineBridge.setClipLook(clip.id, preset, clip.cameraAngle, clip.brightness, clip.contrast, clip.saturation)) {
            syncTimeline()
            statusMessage = "Lens: $preset"
        }
    }

    fun applyColor(brightness: Float, contrast: Float, saturation: Float) {
        val clip = selectedClip ?: return
        if (EngineBridge.setClipLook(clip.id, clip.lensPreset, clip.cameraAngle, brightness, contrast, saturation)) {
            gradeBrightness = brightness
            gradeContrast = contrast
            gradeSaturation = saturation
            syncTimeline()
            refreshPreview()
        }
    }

    fun setCameraAngle(angle: Int) {
        val clip = selectedClip
        if (clip != null) {
            EngineBridge.setClipLook(clip.id, clip.lensPreset, angle, clip.brightness, clip.contrast, clip.saturation)
        }
        if (EngineBridge.switchCameraAngle(angle)) {
            activeCameraAngle = angle
            syncTimeline()
            statusMessage = "Ángulo $angle"
        } else {
            activeCameraAngle = angle
            if (clip != null) syncTimeline()
            statusMessage = "Ángulo $angle asignado al clip"
        }
    }

    fun undo() {
        if (EngineBridge.undo()) {
            syncTimeline()
            loadSuggestions()
            statusMessage = "Undo"
        }
    }

    fun redo() {
        if (EngineBridge.redo()) {
            syncTimeline()
            loadSuggestions()
            statusMessage = "Redo"
        }
    }

    fun loadSuggestions() {
        EngineBridge.analyzeAi()
        suggestions = EngineBridge.aiSuggestions()
    }

    fun executeSuggestion(id: String) {
        val result = runCatching { EngineBridge.executeSuggestion(id) }.getOrElse { it.message ?: "Error" }
        syncTimeline()
        loadSuggestions()
        statusMessage = result
    }

    fun dismissSuggestion(id: String) {
        EngineBridge.dismissSuggestion(id)
        loadSuggestions()
    }

    fun startExport() {
        if (clipCount == 0) {
            statusMessage = "Import a video before exporting"
            return
        }
        exportJob?.cancel()
        exportJob = viewModelScope.launch {
            isExporting = true
            exportStatus = "Exporting…"
            runCatching {
                EngineBridge.startExport()
                var ticks = 0
                while (ticks < 30) {
                    delay(400)
                    val status = EngineBridge.exportStatus()
                    when {
                        status.lastStatus?.contains("Completed", ignoreCase = true) == true -> {
                            val effects = clips.count { it.lensPreset != "none" || it.brightness != 0f || it.contrast != 1f || it.saturation != 1f }
                            exportStatus = buildString {
                                append(status.lastOutputPath?.substringAfterLast('/') ?: "Export complete")
                                if (effects > 0) append(" · look aplicado")
                            }
                            break
                        }
                        status.lastStatus?.contains("Failed", ignoreCase = true) == true -> {
                            exportStatus = "Export failed"
                            break
                        }
                        status.activeExportId != null -> exportStatus = "Exporting…"
                    }
                    ticks++
                }
                if (exportStatus == "Exporting…") {
                    exportStatus = EngineBridge.exportStatus().lastOutputPath?.substringAfterLast('/')
                        ?: if (EngineBridge.useNativeEngine) "Export finished" else "Export complete"
                }
            }.onFailure {
                exportStatus = "Export error: ${it.message}"
            }
            isExporting = false
        }
    }

    private fun syncTimeline() {
        val info = EngineBridge.timelineInfo()
        durationMs = info.durationMs.toFloat().coerceAtLeast(if (info.clipCount > 0) 100f else 1000f)
        playheadMs = info.playheadMs.toFloat().coerceIn(0f, durationMs)
        clipCount = info.clipCount
        clips = info.clips
        info.primaryPath?.let { previewPath = it }
        canUndo = EngineBridge.canUndo()
        canRedo = EngineBridge.canRedo()
        updateSelectedClip()
        updateGradeFromSelected()
        if (clipCount == 0) {
            previewBitmap = null
            previewPath = null
        } else {
            val frame = EngineBridge.scrubTo(playheadMs.toLong())
            decodeTimeMs = frame.sourceTimeMs ?: playheadMs.toLong()
            refreshPreview()
        }
    }

    private fun updateSelectedClip() {
        selectedClipId = clips.find { clip ->
            playheadMs >= clip.startMs && playheadMs < clip.startMs + clip.durationMs
        }?.id
    }

    private fun updateGradeFromSelected() {
        selectedClip?.let {
            gradeBrightness = it.brightness
            gradeContrast = it.contrast
            gradeSaturation = it.saturation
            activeCameraAngle = it.cameraAngle
        }
    }

    private fun refreshPreview() {
        val path = previewPath ?: run {
            previewBitmap = null
            return
        }
        viewModelScope.launch {
            isDecoding = true
            val bytes = decoder.decodeFrame(path, decodeTimeMs, 1280, 720)
            var bitmap = if (bytes.isNotEmpty()) BitmapFactory.decodeByteArray(bytes, 0, bytes.size) else null
            if (bitmap != null && (gradeBrightness != 0f || gradeContrast != 1f || gradeSaturation != 1f)) {
                bitmap = ColorGradeUtil.apply(bitmap, gradeBrightness, gradeContrast, gradeSaturation)
            }
            previewBitmap = bitmap
            isDecoding = false
        }
    }

    override fun onCleared() {
        playbackPause()
        exportJob?.cancel()
        super.onCleared()
    }
}

class HomeViewModel : ViewModel() {
    var projectName by mutableStateOf<String?>(null)
    var bridgeStatus by mutableStateOf(EngineBridge.bridgeStatus())
    var errorMessage by mutableStateOf<String?>(null)

    fun createProject(context: Context, name: String, onCreated: () -> Unit) {
        viewModelScope.launch {
            runCatching {
                val parent = java.io.File(context.filesDir, "projects").apply { mkdirs() }
                ProjectSession.createProject(name, parent.absolutePath)
                projectName = name
                bridgeStatus = EngineBridge.bridgeStatus()
                errorMessage = null
                onCreated()
            }.onFailure {
                errorMessage = it.message
            }
        }
    }
}
