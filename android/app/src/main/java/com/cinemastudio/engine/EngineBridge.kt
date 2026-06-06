package com.cinemastudio.engine

/**
 * Kotlin facade — native Rust when .so linked, else mock timeline editor.
 */
object EngineBridge {
    private val backend: EngineBackend = if (CinemaStudioNative.load()) {
        NativeEngineBackend()
    } else {
        MockEngineBackend()
    }

    val useNativeEngine: Boolean get() = backend is NativeEngineBackend

    fun initialize(dataRoot: String? = null) = backend.initialize(dataRoot)

    fun createProject(name: String, parentDir: String): String =
        backend.createProject(name, parentDir)

    fun openProject(projectDir: String): String = backend.openProject(projectDir)

    fun bridgeStatus(): String = backend.bridgeStatus()

    fun importMedia(sourcePath: String): String = backend.importMedia(sourcePath)

    fun addClipToTimeline(mediaId: String, startMs: Long? = null, sourcePath: String? = null): String =
        backend.addClipToTimeline(mediaId, startMs, sourcePath)

    fun importAndAddClip(sourcePath: String): String {
        val mediaId = importMedia(sourcePath)
        val projectPath = ProjectSession.projectPath
        val vaultPath = if (projectPath != null) {
            java.io.File(projectPath, "media/${java.io.File(sourcePath).name}").absolutePath
        } else {
            sourcePath
        }
        addClipToTimeline(mediaId, sourcePath = vaultPath)
        analyzeAi()
        return mediaId
    }

    fun scrubTo(timeMs: Long): FrameComposition = backend.scrubTo(timeMs)

    fun timelineInfo(): TimelineInfo = backend.timelineInfo()

    fun trimClip(clipId: String, newStartMs: Long, newEndMs: Long): Boolean =
        backend.trimClip(clipId, newStartMs, newEndMs)

    fun splitAtPlayhead(): Boolean = backend.splitAtPlayhead()

    fun duplicateAtPlayhead(): Boolean = backend.duplicateAtPlayhead()

    fun deleteAtPlayhead(): Boolean = backend.deleteAtPlayhead()

    fun setClipLook(
        clipId: String,
        lensPreset: String,
        cameraAngle: Int = 1,
        brightness: Float = 0f,
        contrast: Float = 1f,
        saturation: Float = 1f,
    ): Boolean = backend.setClipLook(clipId, lensPreset, cameraAngle, brightness, contrast, saturation)

    fun switchCameraAngle(angle: Int): Boolean = backend.switchCameraAngle(angle)

    fun undo(): Boolean = backend.undo()

    fun redo(): Boolean = backend.redo()

    fun canUndo(): Boolean = backend.canUndo()

    fun canRedo(): Boolean = backend.canRedo()

    fun analyzeAi() = backend.analyzeAi()

    fun aiSuggestions(): List<AiSuggestionItem> = backend.aiSuggestions()

    fun executeSuggestion(id: String): String = backend.executeSuggestion(id)

    fun dismissSuggestion(id: String) = backend.dismissSuggestion(id)

    fun startExport(width: Int = 1920, height: Int = 1080): String =
        backend.startExport(width, height)

    fun exportStatus(): ExportStatus = backend.exportStatus()
}
