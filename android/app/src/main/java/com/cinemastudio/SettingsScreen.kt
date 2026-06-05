package com.cinemastudio.ui.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.cinemastudio.infrastructure.fetchInfrastructureStatus
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(onBack: () -> Unit, onOpenBeta: () -> Unit) {
    var telemetry by remember { mutableStateOf(false) }
    var crashReports by remember { mutableStateOf(false) }
    var infra by remember { mutableStateOf(com.cinemastudio.infrastructure.InfrastructureStatus()) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        infra = fetchInfrastructureStatus()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings", color = Color.White) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back", tint = Color.White)
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Black)
            )
        },
        containerColor = Color.Black
    ) { padding ->
        Column(
            Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Text("Account", color = Color.White, style = MaterialTheme.typography.titleMedium)
            Text("Optional — core works without login", color = Color.White.copy(0.45f), style = MaterialTheme.typography.bodySmall)

            Text("Cloud backup", color = Color.White, style = MaterialTheme.typography.titleMedium)
            OutlinedButton(onClick = {}, modifier = Modifier.fillMaxWidth()) {
                Text("Backup project", color = Color.White)
            }

            Text("Privacy", color = Color.White, style = MaterialTheme.typography.titleMedium)
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text("Telemetry", color = Color.White)
                Switch(checked = telemetry, onCheckedChange = { telemetry = it })
            }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text("Crash reports", color = Color.White)
                Switch(checked = crashReports, onCheckedChange = { crashReports = it })
            }

            Button(onClick = onOpenBeta, modifier = Modifier.fillMaxWidth()) {
                Text("Beta Program")
            }

            Text("Infrastructure", color = Color.White, style = MaterialTheme.typography.titleMedium)
            Text("GitHub + Vercel live · Supabase pending", color = Color.White.copy(0.45f), style = MaterialTheme.typography.bodySmall)
            InfraRow("GitHub", infra.github)
            InfraRow("Vercel", infra.vercel)
            InfraRow("Supabase", infra.supabase)
            if (infra.note.isNotEmpty()) {
                Text(infra.note, color = Color.White.copy(0.4f), style = MaterialTheme.typography.bodySmall)
            }
            TextButton(onClick = { scope.launch { infra = fetchInfrastructureStatus() } }) {
                Text("Refresh status", color = Color.White.copy(0.7f))
            }
        }
    }
}

@Composable
private fun InfraRow(label: String, value: String) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Text(label, color = Color.White.copy(0.7f))
        Text(value, color = Color.White, style = MaterialTheme.typography.bodySmall)
    }
}
