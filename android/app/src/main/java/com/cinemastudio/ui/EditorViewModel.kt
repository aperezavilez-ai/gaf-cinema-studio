package com.cinemastudio.ui

import android.graphics.BitmapFactory
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.cinemastudio.engine.EngineBridge
import com.cinemastudio.engine.VideoDecoderService
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class EditorViewModel : ViewModel() {
    var playheadMs by mutableStateOf(0f)
    var durationMs by mutableStateOf(5000f)
    var isPlaying by mutableStateOf(false)
    var previewBitmap by mutableStateOf<android.graphics.Bitmap?>(null)
    var isDecoding by mutableStateOf(false)
    var exportStatus by mutableStateOf("")
    var previewPath: String? = null

    private var playbackJob: Job? = null
    private val decoder = VideoDecoderService()

    fun setPreviewPath(path: String?) {
        previewPath = path
        refreshPreview()
    }

    fun scrubTo(ms: Float) {
        playheadMs = ms.coerceIn(0f, durationMs)
        refreshPreview()
    }

    fun playbackPlay() {
        isPlaying = true
        playbackJob?.cancel()
        playbackJob = viewModelScope.launch {
            while (isPlaying) {
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

    fun startExport() {
        viewModelScope.launch {
            exportStatus = "Exporting…"
            delay(800)
            exportStatus = "Export queued (stub)"
        }
    }

    private fun refreshPreview() {
        val path = previewPath ?: run {
            previewBitmap = null
            return
        }
        viewModelScope.launch {
            isDecoding = true
            val bytes = decoder.decodeFrame(path, playheadMs.toLong(), 1280, 720)
            previewBitmap = if (bytes.isNotEmpty()) BitmapFactory.decodeByteArray(bytes, 0, bytes.size) else null
            isDecoding = false
        }
    }

    override fun onCleared() {
        playbackPause()
        super.onCleared()
    }
}

class HomeViewModel : ViewModel() {
    var projectName by mutableStateOf<String?>(null)
    var bridgeStatus by mutableStateOf(EngineBridge.bridgeStatus())

    fun createProject(name: String) {
        projectName = name
        bridgeStatus = EngineBridge.bridgeStatus()
    }
}
