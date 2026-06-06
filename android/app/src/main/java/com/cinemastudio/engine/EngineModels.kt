package com.cinemastudio.engine

import org.json.JSONArray
import org.json.JSONObject

data class FrameComposition(
    val timeMs: Long,
    val videoLayerCount: Int,
    val primaryPath: String?,
    val usesProxy: Boolean,
    val sourceTimeMs: Long?,
)

data class TimelineClip(
    val id: String,
    val startMs: Long,
    val durationMs: Long,
    val fadeInMs: Long,
    val fadeOutMs: Long,
    val label: String,
    val lensPreset: String = "none",
    val cameraAngle: Int = 1,
    val brightness: Float = 0f,
    val contrast: Float = 1f,
    val saturation: Float = 1f,
)

data class TimelineInfo(
    val durationMs: Long,
    val playheadMs: Long,
    val clipCount: Int,
    val primaryPath: String?,
    val clips: List<TimelineClip>,
)

data class ExportStatus(
    val activeExportId: String?,
    val lastStatus: String?,
    val lastOutputPath: String?,
    val ffmpegAvailable: Boolean,
)

data class AiSuggestionItem(
    val id: String,
    val message: String,
    val priority: String,
    val actionLabel: String?,
    val isActionable: Boolean,
)

object EngineJson {
    fun parseFrame(json: String): FrameComposition {
        val o = JSONObject(json)
        return FrameComposition(
            timeMs = o.optLong("timeMs"),
            videoLayerCount = o.optInt("videoLayerCount"),
            primaryPath = o.optString("primaryPath").takeIf { it.isNotEmpty() },
            usesProxy = o.optBoolean("usesProxy"),
            sourceTimeMs = if (o.has("sourceTimeMs") && !o.isNull("sourceTimeMs")) {
                o.getLong("sourceTimeMs")
            } else {
                null
            },
        )
    }

    fun parseTimelineInfo(json: String): TimelineInfo {
        val o = JSONObject(json)
        val clips = mutableListOf<TimelineClip>()
        o.optJSONArray("clips")?.let { arr ->
            for (i in 0 until arr.length()) {
                val c = arr.getJSONObject(i)
                clips.add(
                    TimelineClip(
                        id = c.getString("id"),
                        startMs = c.getLong("startMs"),
                        durationMs = c.getLong("durationMs"),
                        fadeInMs = c.optLong("fadeInMs"),
                        fadeOutMs = c.optLong("fadeOutMs"),
                        label = c.optString("label", "Clip"),
                        lensPreset = c.optString("lensPreset", "none"),
                        cameraAngle = c.optInt("cameraAngle", 1),
                        brightness = c.optDouble("brightness", 0.0).toFloat(),
                        contrast = c.optDouble("contrast", 1.0).toFloat(),
                        saturation = c.optDouble("saturation", 1.0).toFloat(),
                    )
                )
            }
        }
        return TimelineInfo(
            durationMs = o.optLong("durationMs"),
            playheadMs = o.optLong("playheadMs"),
            clipCount = o.optInt("clipCount"),
            primaryPath = o.optString("primaryPath").takeIf { it.isNotEmpty() },
            clips = clips,
        )
    }

    fun parseExportStatus(json: String): ExportStatus {
        val o = JSONObject(json)
        return ExportStatus(
            activeExportId = o.optString("activeExportId").takeIf { it.isNotEmpty() && it != "null" },
            lastStatus = o.optString("lastStatus").takeIf { it.isNotEmpty() && it != "null" },
            lastOutputPath = o.optString("lastOutputPath").takeIf { it.isNotEmpty() && it != "null" },
            ffmpegAvailable = o.optBoolean("ffmpegAvailable"),
        )
    }

    fun parseAiSuggestions(json: String): List<AiSuggestionItem> {
        val arr = JSONArray(json)
        val items = mutableListOf<AiSuggestionItem>()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            items.add(
                AiSuggestionItem(
                    id = o.getString("id"),
                    message = o.getString("message"),
                    priority = o.optString("priority", "medium"),
                    actionLabel = o.optString("actionLabel").takeIf { it.isNotEmpty() },
                    isActionable = o.optBoolean("isActionable"),
                )
            )
        }
        return items
    }
}
