package com.cinemastudio.engine

/**
 * Kotlin bridge to Rust engine via UniFFI/JNI.
 * Set [useNativeEngine] = true after generating bindings with scripts/generate_bindings.sh
 */
object EngineBridge {
    var useNativeEngine: Boolean = false

    fun initialize(dataRoot: String? = null) {
        if (!useNativeEngine) return
        // csEngineInit(dataRoot)
    }

    fun createProject(name: String, parentDir: String): String {
        if (useNativeEngine) {
            // return csCreateProject(name, parentDir)
            error("UniFFI bindings not linked")
        }
        val safe = name.replace(" ", "_")
        return "$parentDir/$safe.csproj"
    }

    fun openProject(projectDir: String): String {
        if (useNativeEngine) {
            // return csOpenProject(projectDir)
            error("UniFFI bindings not linked")
        }
        return projectDir.substringAfterLast("/").removeSuffix(".csproj")
    }

    fun bridgeStatus(): String =
        if (useNativeEngine) "{}" else """{"mode":"mock"}"""
}
