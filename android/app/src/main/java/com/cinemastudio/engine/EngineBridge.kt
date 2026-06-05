package com.cinemastudio.engine

/**
 * Kotlin facade — native Rust when .so linked, else mock.
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
}
