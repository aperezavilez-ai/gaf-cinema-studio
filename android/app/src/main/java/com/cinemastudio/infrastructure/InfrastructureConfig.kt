package com.cinemastudio.infrastructure

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

object InfrastructureConfig {
    const val CUSTOM_BASE = "https://gafcinemastudio.com"
    const val CUSTOM_WWW_BASE = "https://www.gafcinemastudio.com"
    const val VERCEL_BASE = "https://gaf-cinema-studio.vercel.app"
    const val GITHUB_REPO = "aperezavilez-ai/gaf-cinema-studio"
}

data class InfrastructureStatus(
    val github: String = "linked",
    val vercel: String = "checking…",
    val supabase: String = "pending",
    val note: String = "",
)

suspend fun fetchInfrastructureStatus(): InfrastructureStatus = withContext(Dispatchers.IO) {
    fetchFrom(InfrastructureConfig.CUSTOM_BASE)
        ?: fetchFrom(InfrastructureConfig.CUSTOM_WWW_BASE)?.copy(
            note = "Using www — configure apex DNS for gafcinemastudio.com",
        )
        ?: fetchFrom(InfrastructureConfig.VERCEL_BASE)?.copy(
            vercel = "deployed (fallback)",
            note = "Custom domain unreachable — using vercel.app",
        )
        ?: InfrastructureStatus(vercel = "offline", note = "Could not reach Vercel")
}

private fun fetchFrom(base: String): InfrastructureStatus? {
    return try {
        val conn = URL("$base/api/status").openConnection() as HttpURLConnection
        conn.requestMethod = "GET"
        conn.connectTimeout = 10_000
        conn.readTimeout = 10_000
        val body = conn.inputStream.bufferedReader().readText()
        conn.disconnect()
        val json = JSONObject(body)
        val connections = json.getJSONObject("connections")
        InfrastructureStatus(
            github = connections.getJSONObject("github").getString("status"),
            vercel = connections.getJSONObject("vercel").getString("status"),
            supabase = connections.getJSONObject("supabase").getString("status"),
            note = connections.getJSONObject("supabase").optString("note", ""),
        )
    } catch (_: Exception) {
        null
    }
}
