package com.cinemastudio.engine

/**
 * MediaCodec decode service — wire to Rust native_bridge at integration.
 */
class VideoDecoderService {
    fun decodeFrame(path: String, timeMs: Long, width: Int, height: Int): ByteArray {
        // TODO: MediaExtractor + MediaCodec pipeline
        return ByteArray(0)
    }

    fun registerWithEngine() {
        // TODO: JNI callback registration
    }
}
