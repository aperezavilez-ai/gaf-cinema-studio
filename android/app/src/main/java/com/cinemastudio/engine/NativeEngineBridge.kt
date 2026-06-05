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
    external fun csCScrubTo(timeMs: Long): String
    external fun csCSetDecoderBackend(name: String): Int

    @JvmStatic
    private fun registerNatives() {
        // Populated when UniFFI/JNI generated bindings are linked
    }
}

interface EngineBackend {
    fun initialize(dataRoot: String?)
    fun createProject(name: String, parentDir: String): String
    fun openProject(projectDir: String): String
    fun bridgeStatus(): String
}

class MockEngineBackend : EngineBackend {
    override fun initialize(dataRoot: String?) {}
    override fun createProject(name: String, parentDir: String): String {
        val safe = name.replace(" ", "_")
        return "$parentDir/$safe.csproj"
    }
    override fun openProject(projectDir: String): String =
        projectDir.substringAfterLast("/").removeSuffix(".csproj")
    override fun bridgeStatus(): String = """{"mode":"mock"}"""
}

class NativeEngineBackend : EngineBackend {
    init {
        require(CinemaStudioNative.load()) { "libcinemastudio_engine.so not loaded" }
    }

    override fun initialize(dataRoot: String?) {
        CinemaStudioNative.csCEngineInit(dataRoot)
    }

    override fun createProject(name: String, parentDir: String): String {
        val result = CinemaStudioNative.csCCreateProject(name, parentDir)
        if (result.startsWith("ERROR:")) error(result.removePrefix("ERROR:"))
        return result
    }

    override fun openProject(projectDir: String): String {
        val result = CinemaStudioNative.csCOpenProject(projectDir)
        if (result.startsWith("ERROR:")) error(result.removePrefix("ERROR:"))
        return result
    }

    override fun bridgeStatus(): String = CinemaStudioNative.csCBridgeStatus()
}
