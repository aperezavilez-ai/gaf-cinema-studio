package com.cinemastudio.engine

import java.io.File
import java.util.UUID

internal data class MockClip(
    val id: String,
    val path: String,
    var startMs: Long,
    var durationMs: Long,
    var sourceInMs: Long,
    var mediaDurationMs: Long,
    var fadeInMs: Long = 0,
    var fadeOutMs: Long = 0,
    var label: String = "Clip",
    var lensPreset: String = "none",
    var cameraAngle: Int = 1,
    var brightness: Float = 0f,
    var contrast: Float = 1f,
    var saturation: Float = 1f,
)

internal class MockTimelineState {
    var projectPath: String? = null
    val clips = mutableListOf<MockClip>()
    private val undoStack = ArrayDeque<List<MockClip>>()
    private val redoStack = ArrayDeque<List<MockClip>>()
    var playheadMs: Long = 0
    var activeCameraAngle: Int = 1
    val dismissedSuggestions = mutableSetOf<String>()
    var suggestions: List<AiSuggestionItem> = emptyList()

    fun timelineDurationMs(): Long =
        clips.maxOfOrNull { it.startMs + it.durationMs } ?: 0L

    fun clipAtPlayhead(): MockClip? {
        val at = clips.filter { playheadMs >= it.startMs && playheadMs < it.startMs + it.durationMs }
        return at.find { it.cameraAngle == activeCameraAngle } ?: at.firstOrNull()
    }

    fun anyClipAtPlayhead(): MockClip? =
        clips.find { playheadMs >= it.startMs && playheadMs < it.startMs + it.durationMs }

    fun clipById(id: String): MockClip? = clips.find { it.id == id }

    fun primaryPath(): String? =
        clipAtPlayhead()?.path ?: clips.minByOrNull { it.startMs }?.path

    fun sourceTimeAtPlayhead(): Long? {
        val clip = clipAtPlayhead() ?: return null
        return clip.sourceInMs + (playheadMs - clip.startMs)
    }

    fun pushUndo() {
        undoStack.addLast(clips.map { it.copy() })
        redoStack.clear()
        if (undoStack.size > 50) undoStack.removeFirst()
    }

    fun undo(): Boolean {
        if (undoStack.isEmpty()) return false
        redoStack.addLast(clips.map { it.copy() })
        clips.clear()
        clips.addAll(undoStack.removeLast())
        return true
    }

    fun redo(): Boolean {
        if (redoStack.isEmpty()) return false
        undoStack.addLast(clips.map { it.copy() })
        clips.clear()
        clips.addAll(redoStack.removeLast())
        return true
    }

    fun canUndo(): Boolean = undoStack.isNotEmpty()
    fun canRedo(): Boolean = redoStack.isNotEmpty()

    fun reset() {
        projectPath = null
        clips.clear()
        undoStack.clear()
        redoStack.clear()
        playheadMs = 0
        suggestions = emptyList()
        dismissedSuggestions.clear()
    }

    fun toTimelineClips(): List<TimelineClip> =
        clips.sortedBy { it.startMs }.map {
            TimelineClip(
                id = it.id,
                startMs = it.startMs,
                durationMs = it.durationMs,
                fadeInMs = it.fadeInMs,
                fadeOutMs = it.fadeOutMs,
                label = it.label,
                lensPreset = it.lensPreset,
                cameraAngle = it.cameraAngle,
                brightness = it.brightness,
                contrast = it.contrast,
                saturation = it.saturation,
            )
        }
}

internal object MockAiRules {
    private const val LONG_CLIP_MS = 60_000L
    private const val DEFAULT_FADE_MS = 300L

    fun analyze(state: MockTimelineState): List<AiSuggestionItem> {
        val items = mutableListOf<AiSuggestionItem>()
        if (state.clips.isEmpty()) {
            items.add(suggestion("Importa tu primer clip para comenzar la producción.", "hint_import", false))
        }
        state.clips.firstOrNull { it.durationMs > LONG_CLIP_MS }?.let { clip ->
            items.add(
                suggestion(
                    "El clip \"${clip.label}\" dura ${"%.0f".format(clip.durationMs / 1000.0)}s. Considera dividirlo.",
                    "hint_split",
                    true,
                )
            )
        }
        if (state.clips.isNotEmpty() && state.clips.any { it.fadeInMs == 0L && it.fadeOutMs == 0L }) {
            items.add(
                suggestion(
                    "Aplica fades cinematográficos (300ms) a tus clips.",
                    "apply_default_fades",
                    true,
                )
            )
        }
        if (state.clips.isNotEmpty()) {
            items.add(
                suggestion(
                    "Timeline lista. ¿Exportar MP4 1080p?",
                    "start_export",
                    true,
                )
            )
        }
        if state.clips.size >= 2 {
            val grouped = state.clips.groupBy { it.startMs }
            if (grouped.values.any { it.size >= 2 }) {
                items.add(
                    suggestion(
                        "Tomas multicámara detectadas. Usa Ángulo 1/2/3 para alternar.",
                        "hint_multicam",
                        false,
                    )
                )
            }
        }
        return items.filter { it.id !in state.dismissedSuggestions }
    }

    private fun suggestion(message: String, actionId: String, actionable: Boolean): AiSuggestionItem {
        val id = UUID.nameUUIDFromBytes("$message$actionId".toByteArray()).toString()
        return AiSuggestionItem(
            id = id,
            message = message,
            priority = "medium",
            actionLabel = if (actionable) "Ejecutar" else null,
            isActionable = actionable,
        )
    }

    fun execute(state: MockTimelineState, suggestion: AiSuggestionItem): String {
        return when {
            suggestion.message.contains("dividirlo") -> {
                if (state.clipAtPlayhead() != null || state.clips.isNotEmpty()) {
                    "Coloca el playhead dentro del clip y usa Split"
                } else {
                    "No hay clips"
                }
            }
            suggestion.message.contains("fades") -> {
                state.pushUndo()
                state.clips.forEach {
                    it.fadeInMs = DEFAULT_FADE_MS
                    it.fadeOutMs = DEFAULT_FADE_MS
                }
                "Fades 300ms aplicados a ${state.clips.size} clip(s)"
            }
            suggestion.message.contains("Exportar") -> "Usa el botón Export en la barra de herramientas"
            else -> "OK"
        }
    }
}

internal class MockEngineBackend : EngineBackend {
    private val state = MockTimelineState()

    override fun initialize(dataRoot: String?) {}

    override fun createProject(name: String, parentDir: String): String {
        val safe = name.replace(" ", "_").replace(Regex("[^A-Za-z0-9_-]"), "")
        val base = if (safe.isEmpty()) "Project" else safe
        var dirName = "$base.csproj"
        var n = 1
        while (File(parentDir, dirName).exists()) {
            dirName = "${base}_$n.csproj"
            n++
        }
        val projectDir = File(parentDir, dirName)
        projectDir.mkdirs()
        File(projectDir, "media").mkdirs()
        File(projectDir, "exports").mkdirs()
        File(projectDir, "project.json").writeText(
            """{"schemaVersion":1,"metadata":{"name":"$name"}}"""
        )
        state.reset()
        state.projectPath = projectDir.absolutePath
        refreshAi()
        return projectDir.absolutePath
    }

    override fun openProject(projectDir: String): String {
        state.projectPath = projectDir
        state.clips.clear()
        state.undoStack.clear()
        state.redoStack.clear()
        state.playheadMs = 0
        loadClipsFromMediaDir(projectDir)
        refreshAi()
        return File(projectDir).name.removeSuffix(".csproj")
    }

    override fun bridgeStatus(): String = """{"mode":"mock","androidPreview":true}"""

    override fun importMedia(sourcePath: String): String {
        val projectPath = state.projectPath ?: error("No open project")
        val mediaDir = File(projectPath, "media").apply { mkdirs() }
        val source = File(sourcePath)
        val dest = File(mediaDir, source.name)
        if (source.absolutePath != dest.absolutePath) {
            source.copyTo(dest, overwrite = true)
        }
        return UUID.randomUUID().toString()
    }

    override fun addClipToTimeline(mediaId: String, startMs: Long?, sourcePath: String?): String {
        val projectPath = state.projectPath ?: error("No open project")
        val videoPath = sourcePath ?: run {
            val mediaDir = File(projectPath, "media")
            mediaDir.listFiles()?.firstOrNull {
                it.isFile && (it.extension.equals("mp4", true) || it.extension.equals("mov", true))
            }?.absolutePath ?: error("No media in project")
        }
        val mediaDuration = MediaHelper.durationMs(videoPath).coerceAtLeast(1000L)
        state.pushUndo()
        val clip = MockClip(
            id = UUID.randomUUID().toString(),
            path = videoPath,
            startMs = startMs ?: state.timelineDurationMs(),
            durationMs = mediaDuration,
            sourceInMs = 0,
            mediaDurationMs = mediaDuration,
            label = File(videoPath).nameWithoutExtension,
        )
        state.clips.add(clip)
        state.clips.sortBy { it.startMs }
        refreshAi()
        return clip.id
    }

    override fun scrubTo(timeMs: Long): FrameComposition {
        val duration = state.timelineDurationMs().coerceAtLeast(1L)
        state.playheadMs = timeMs.coerceIn(0, duration)
        return FrameComposition(
            timeMs = state.playheadMs,
            videoLayerCount = if (state.clips.isEmpty()) 0 else 1,
            primaryPath = state.primaryPath(),
            usesProxy = false,
            sourceTimeMs = state.sourceTimeAtPlayhead(),
        )
    }

    override fun timelineInfo(): TimelineInfo {
        val duration = state.timelineDurationMs()
        return TimelineInfo(
            durationMs = duration,
            playheadMs = state.playheadMs.coerceIn(0, duration.coerceAtLeast(1L)),
            clipCount = state.clips.size,
            primaryPath = state.primaryPath(),
            clips = state.toTimelineClips(),
        )
    }

    override fun trimClip(clipId: String, newStartMs: Long, newEndMs: Long): Boolean {
        val clip = state.clipById(clipId) ?: return false
        val clipEnd = clip.startMs + clip.durationMs
        if (newStartMs >= newEndMs || newEndMs > clipEnd || newEndMs - newStartMs < 500) return false
        state.pushUndo()
        if (newStartMs > clip.startMs) {
            val delta = newStartMs - clip.startMs
            clip.sourceInMs += delta
            clip.startMs = newStartMs
            clip.durationMs -= delta
        }
        clip.durationMs = newEndMs - clip.startMs
        refreshAi()
        return true
    }

    override fun splitAtPlayhead(): Boolean {
        val clip = state.anyClipAtPlayhead() ?: return false
        val offset = state.playheadMs - clip.startMs
        if (offset <= 0 || offset >= clip.durationMs) return false
        state.pushUndo()
        val right = clip.copy(
            id = UUID.randomUUID().toString(),
            startMs = state.playheadMs,
            durationMs = clip.durationMs - offset,
            sourceInMs = clip.sourceInMs + offset,
            label = "${clip.label} B",
        )
        clip.durationMs = offset
        clip.label = "${clip.label} A"
        state.clips.add(right)
        state.clips.sortBy { it.startMs }
        refreshAi()
        return true
    }

    override fun deleteAtPlayhead(): Boolean {
        val index = state.clips.indexOfFirst {
            state.playheadMs >= it.startMs && state.playheadMs < it.startMs + it.durationMs
        }
        if (index < 0) return false
        state.pushUndo()
        state.clips.removeAt(index)
        val duration = state.timelineDurationMs()
        if (state.playheadMs > duration) state.playheadMs = duration
        refreshAi()
        return true
    }

    override fun duplicateAtPlayhead(): Boolean {
        val clip = state.anyClipAtPlayhead() ?: return false
        state.pushUndo()
        val dup = clip.copy(
            id = UUID.randomUUID().toString(),
            startMs = clip.startMs + clip.durationMs,
            label = "${clip.label} (copy)",
        )
        state.clips.add(dup)
        state.clips.sortBy { it.startMs }
        refreshAi()
        return true
    }

    override fun setClipLook(
        clipId: String,
        lensPreset: String,
        cameraAngle: Int,
        brightness: Float,
        contrast: Float,
        saturation: Float,
    ): Boolean {
        val clip = state.clipById(clipId) ?: return false
        state.pushUndo()
        clip.lensPreset = lensPreset
        clip.cameraAngle = cameraAngle.coerceIn(1, 3)
        clip.brightness = brightness.coerceIn(-1f, 1f)
        clip.contrast = contrast.coerceIn(0f, 2f)
        clip.saturation = saturation.coerceIn(0f, 2f)
        refreshAi()
        return true
    }

    override fun switchCameraAngle(angle: Int): Boolean {
        state.activeCameraAngle = angle.coerceIn(1, 3)
        val playhead = state.playheadMs
        val anchor = state.clips.find { playhead >= it.startMs && playhead < it.startMs + it.durationMs }
        val anchorStart = anchor?.startMs
        val target = state.clips.find { clip ->
            clip.cameraAngle == state.activeCameraAngle &&
                (anchorStart?.let { clip.startMs == it }
                    ?: (playhead >= clip.startMs && playhead < clip.startMs + clip.durationMs))
        } ?: state.clips.filter { it.cameraAngle == state.activeCameraAngle && it.startMs >= playhead }
            .minByOrNull { it.startMs }
        if (target != null) {
            val offset = if (playhead >= target.startMs && playhead < target.startMs + target.durationMs) {
                playhead - target.startMs
            } else {
                0L
            }
            state.playheadMs = target.startMs + offset
            return true
        }
        return false
    }

    override fun undo(): Boolean {
        val ok = state.undo()
        if (ok) refreshAi()
        return ok
    }

    override fun redo(): Boolean {
        val ok = state.redo()
        if (ok) refreshAi()
        return ok
    }

    override fun canUndo(): Boolean = state.canUndo()
    override fun canRedo(): Boolean = state.canRedo()

    override fun analyzeAi() {
        refreshAi()
    }

    override fun aiSuggestions(): List<AiSuggestionItem> = state.suggestions

    override fun executeSuggestion(id: String): String {
        val item = state.suggestions.find { it.id == id } ?: return "Suggestion not found"
        val result = MockAiRules.execute(state, item)
        state.dismissedSuggestions.add(id)
        refreshAi()
        return result
    }

    override fun dismissSuggestion(id: String) {
        state.dismissedSuggestions.add(id)
        refreshAi()
    }

    override fun startExport(width: Int, height: Int): String {
        val projectPath = state.projectPath ?: error("No open project")
        val exportsDir = File(projectPath, "exports").apply { mkdirs() }
        val fadeCount = state.clips.count { it.fadeInMs > 0 || it.fadeOutMs > 0 }
        val suffix = if (fadeCount > 0) "_fades" else ""
        val out = File(exportsDir, "export_${System.currentTimeMillis()}$suffix.mp4")
        val sorted = state.clips.sortedBy { it.startMs }
        val source = sorted.firstOrNull()?.path
            ?: File(projectPath, "media").listFiles()?.firstOrNull()?.absolutePath
        if (source != null) {
            File(source).copyTo(out, overwrite = true)
        } else {
            out.writeText("mock export — import a video first")
        }
        lastExportPath = out.absolutePath
        lastExportStatus = "Completed"
        lastExportFadeNote = if (fadeCount > 0) {
            "$fadeCount clip(s) with 300ms fades (native engine renders fades in MP4)"
        } else null
        return UUID.randomUUID().toString()
    }

    override fun exportStatus(): ExportStatus = ExportStatus(
        activeExportId = null,
        lastStatus = lastExportStatus,
        lastOutputPath = lastExportPath,
        ffmpegAvailable = false,
    )

    fun lastFadeNote(): String? = lastExportFadeNote

    private var lastExportPath: String? = null
    private var lastExportStatus: String? = null
    private var lastExportFadeNote: String? = null

    private fun refreshAi() {
        state.suggestions = MockAiRules.analyze(state)
    }

    private fun loadClipsFromMediaDir(projectDir: String) {
        val mediaDir = File(projectDir, "media")
        val video = mediaDir.listFiles()?.firstOrNull {
            it.isFile && (it.extension.equals("mp4", true) || it.extension.equals("mov", true))
        } ?: return
        val duration = MediaHelper.durationMs(video.absolutePath).coerceAtLeast(1000L)
        state.clips.add(
            MockClip(
                id = UUID.randomUUID().toString(),
                path = video.absolutePath,
                startMs = 0,
                durationMs = duration,
                sourceInMs = 0,
                mediaDurationMs = duration,
                label = video.nameWithoutExtension,
            )
        )
    }
}
