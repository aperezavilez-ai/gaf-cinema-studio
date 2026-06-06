package com.cinemastudio.engine

/**
 * JNI/C ABI bindings to Rust engine — loaded when .so is present in jniLibs.
 */
object CinemaStudioNative {
    private var loaded = false

    fun load(): Boolean {
        if (loaded) return true
        return try {
            System.loadLibrary("cinemastudio_engine")
            loaded = true
            true
        } catch (_: UnsatisfiedLinkError) {
            false
        }
    }

    external fun csCEngineInit(dataRoot: String?): Int
    external fun csCBridgeStatus(): String
    external fun csCCreateProject(name: String, parentDir: String): String
    external fun csCOpenProject(projectDir: String): String
    external fun csCSaveProject(): Int
    external fun csCImportMedia(sourcePath: String): String
    external fun csCAddClip(mediaId: String, startMs: Long): String
    external fun csCScrubTo(timeMs: Long): String
    external fun csCTimelineInfo(): String
    external fun csCTrimClip(clipId: String, newStartMs: Long, newEndMs: Long): Int
    external fun csCSplitAtPlayhead(): Int
    external fun csCDuplicateAtPlayhead(): Int
    external fun csCDeleteAtPlayhead(): Int
    external fun csCSetClipLook(json: String): Int
    external fun csCSwitchCameraAngle(angle: Int): Int
    external fun csCUndo(): Int
    external fun csCRedo(): Int
    external fun csCCanUndo(): Int
    external fun csCCanRedo(): Int
    external fun csCAiSuggestions(): String
    external fun csCAiAnalyze(): Int
    external fun csCAiExecute(suggestionId: String): String
    external fun csCAiDismiss(suggestionId: String): Int
    external fun csCStartExport(width: Int, height: Int, frameRate: Double): String
    external fun csCExportStatus(): String
    external fun csCSetDecoderBackend(name: String): Int
}

interface EngineBackend {
    fun initialize(dataRoot: String?)
    fun createProject(name: String, parentDir: String): String
    fun openProject(projectDir: String): String
    fun bridgeStatus(): String
    fun importMedia(sourcePath: String): String
    fun addClipToTimeline(mediaId: String, startMs: Long? = null, sourcePath: String? = null): String
    fun scrubTo(timeMs: Long): FrameComposition
    fun timelineInfo(): TimelineInfo
    fun trimClip(clipId: String, newStartMs: Long, newEndMs: Long): Boolean
    fun splitAtPlayhead(): Boolean
    fun duplicateAtPlayhead(): Boolean
    fun deleteAtPlayhead(): Boolean
    fun setClipLook(
        clipId: String,
        lensPreset: String,
        cameraAngle: Int,
        brightness: Float,
        contrast: Float,
        saturation: Float,
    ): Boolean
    fun switchCameraAngle(angle: Int): Boolean
    fun undo(): Boolean
    fun redo(): Boolean
    fun canUndo(): Boolean
    fun canRedo(): Boolean
    fun analyzeAi()
    fun aiSuggestions(): List<AiSuggestionItem>
    fun executeSuggestion(id: String): String
    fun dismissSuggestion(id: String)
    fun startExport(width: Int = 1920, height: Int = 1080): String
    fun exportStatus(): ExportStatus
}

class NativeEngineBackend : EngineBackend {
    init {
        require(CinemaStudioNative.load()) { "libcinemastudio_engine.so not loaded" }
    }

    override fun initialize(dataRoot: String?) {
        CinemaStudioNative.csCEngineInit(dataRoot)
        CinemaStudioNative.csCSetDecoderBackend("mediacodec")
    }

    override fun createProject(name: String, parentDir: String): String =
        requireString(CinemaStudioNative.csCCreateProject(name, parentDir))

    override fun openProject(projectDir: String): String =
        requireString(CinemaStudioNative.csCOpenProject(projectDir))

    override fun bridgeStatus(): String = CinemaStudioNative.csCBridgeStatus()

    override fun importMedia(sourcePath: String): String =
        requireString(CinemaStudioNative.csCImportMedia(sourcePath))

    override fun addClipToTimeline(mediaId: String, startMs: Long?, sourcePath: String?): String =
        requireString(CinemaStudioNative.csCAddClip(mediaId, startMs ?: -1L))

    override fun scrubTo(timeMs: Long): FrameComposition =
        EngineJson.parseFrame(requireString(CinemaStudioNative.csCScrubTo(timeMs)))

    override fun timelineInfo(): TimelineInfo =
        EngineJson.parseTimelineInfo(requireString(CinemaStudioNative.csCTimelineInfo()))

    override fun trimClip(clipId: String, newStartMs: Long, newEndMs: Long): Boolean =
        CinemaStudioNative.csCTrimClip(clipId, newStartMs, newEndMs) == 1

    override fun splitAtPlayhead(): Boolean = CinemaStudioNative.csCSplitAtPlayhead() == 1

    override fun duplicateAtPlayhead(): Boolean = CinemaStudioNative.csCDuplicateAtPlayhead() == 1

    override fun deleteAtPlayhead(): Boolean = CinemaStudioNative.csCDeleteAtPlayhead() == 1

    override fun setClipLook(
        clipId: String,
        lensPreset: String,
        cameraAngle: Int,
        brightness: Float,
        contrast: Float,
        saturation: Float,
    ): Boolean {
        val json = org.json.JSONObject()
            .put("clipId", clipId)
            .put("lensPreset", lensPreset)
            .put("cameraAngle", cameraAngle)
            .put("brightness", brightness.toDouble())
            .put("contrast", contrast.toDouble())
            .put("saturation", saturation.toDouble())
            .toString()
        return CinemaStudioNative.csCSetClipLook(json) == 1
    }

    override fun switchCameraAngle(angle: Int): Boolean =
        CinemaStudioNative.csCSwitchCameraAngle(angle) == 1

    override fun undo(): Boolean = CinemaStudioNative.csCUndo() == 1

    override fun redo(): Boolean = CinemaStudioNative.csCRedo() == 1

    override fun canUndo(): Boolean = CinemaStudioNative.csCCanUndo() == 1

    override fun canRedo(): Boolean = CinemaStudioNative.csCCanRedo() == 1

    override fun analyzeAi() {
        CinemaStudioNative.csCAiAnalyze()
    }

    override fun aiSuggestions(): List<AiSuggestionItem> =
        EngineJson.parseAiSuggestions(requireString(CinemaStudioNative.csCAiSuggestions()))

    override fun executeSuggestion(id: String): String =
        requireString(CinemaStudioNative.csCAiExecute(id))

    override fun dismissSuggestion(id: String) {
        CinemaStudioNative.csCAiDismiss(id)
    }

    override fun startExport(width: Int, height: Int): String =
        requireString(CinemaStudioNative.csCStartExport(width, height, 24.0))

    override fun exportStatus(): ExportStatus =
        EngineJson.parseExportStatus(requireString(CinemaStudioNative.csCExportStatus()))

    private fun requireString(result: String): String {
        if (result.startsWith("ERROR:")) error(result.removePrefix("ERROR:"))
        return result
    }
}
