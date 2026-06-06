package com.cinemastudio.engine

object ProjectSession {
    var projectPath: String? = null
        private set
    var projectName: String? = null
        private set

    fun hasProject(): Boolean = projectPath != null

    fun createProject(name: String, parentDir: String): String {
        val path = EngineBridge.createProject(name, parentDir)
        projectPath = path
        projectName = name
        return path
    }

    fun openProject(path: String): String {
        val name = EngineBridge.openProject(path)
        projectPath = path
        projectName = name
        return name
    }

    fun clear() {
        projectPath = null
        projectName = null
    }
}
