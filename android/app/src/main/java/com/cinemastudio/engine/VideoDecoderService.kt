package com.cinemastudio.engine

import android.graphics.Bitmap
import android.media.MediaMetadataRetriever
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** MediaCodec/Retriever decode — wire to Rust native_bridge at integration. */
class VideoDecoderService {
    suspend fun decodeFrame(path: String, timeMs: Long, width: Int, height: Int): ByteArray =
        withContext(Dispatchers.IO) {
            runCatching {
                MediaMetadataRetriever().use { retriever ->
                    retriever.setDataSource(path)
                    val frame = retriever.getFrameAtTime(
                        timeMs * 1000,
                        MediaMetadataRetriever.OPTION_CLOSEST
                    ) ?: return@withContext ByteArray(0)
                    val scaled = Bitmap.createScaledBitmap(frame, width, height, true)
                    val stream = java.io.ByteArrayOutputStream()
                    scaled.compress(Bitmap.CompressFormat.PNG, 90, stream)
                    stream.toByteArray()
                }
            }.getOrDefault(ByteArray(0))
        }

    fun registerWithEngine() {
        // TODO: JNI callback registration
    }
}
