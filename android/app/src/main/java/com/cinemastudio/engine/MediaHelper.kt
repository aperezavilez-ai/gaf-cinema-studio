package com.cinemastudio.engine

import android.content.Context
import android.media.MediaMetadataRetriever
import android.net.Uri
import java.io.File
import java.util.UUID

object MediaHelper {
    fun durationMs(path: String): Long {
        return runCatching {
            MediaMetadataRetriever().use { retriever ->
                retriever.setDataSource(path)
                retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)?.toLong() ?: 0L
            }
        }.getOrDefault(0L)
    }

    fun copyUriToCache(context: Context, uri: Uri): String {
        val name = "import_${UUID.randomUUID()}.mp4"
        val dest = File(context.cacheDir, name)
        context.contentResolver.openInputStream(uri)?.use { input ->
            dest.outputStream().use { output -> input.copyTo(output) }
        } ?: error("Cannot read selected video")
        return dest.absolutePath
    }
}
