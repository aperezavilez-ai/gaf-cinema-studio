package com.cinemastudio.infrastructure

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

object InfrastructureConfig {
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
    try {
        val conn = URL("${InfrastructureConfig.VERCEL_BASE}/api/status").openConnection() as HttpURLConnection
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
        InfrastructureStatus(vercel = "offline", note = "Could not reach Vercel")
    }
}
