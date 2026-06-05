package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cinemastudio.engine.EngineBridge

@Composable
fun HomeScreen(onOpenEditor: () -> Unit, onOpenSettings: () -> Unit) {
    var projectName by remember { mutableStateOf<String?>(null) }
    var bridgeStatus by remember { mutableStateOf(EngineBridge.bridgeStatus()) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        Spacer(modifier = Modifier.height(32.dp))

        Text(
            "CINEMASTUDIO",
            color = Color.White,
            fontSize = 24.sp,
            fontWeight = FontWeight.Light,
            letterSpacing = 4.sp
        )
        Text(
            "Mobile cinematic studio",
            color = Color.White.copy(alpha = 0.5f),
            style = MaterialTheme.typography.bodyMedium
        )

        projectName?.let { name ->
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(containerColor = Color.White.copy(alpha = 0.06f))
            ) {
                Column(Modifier.padding(16.dp)) {
                    Text(name, color = Color.White, style = MaterialTheme.typography.titleMedium)
                    TextButton(onClick = onOpenEditor) {
                        Text("Open editor →", color = Color.White.copy(alpha = 0.7f))
                    }
                }
            }
        }

        Button(
            onClick = {
                projectName = "Demo Film"
                bridgeStatus = EngineBridge.bridgeStatus()
            },
            modifier = Modifier.fillMaxWidth(),
            colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
        ) {
            Text("New Project")
        }

        OutlinedButton(onClick = onOpenSettings, modifier = Modifier.fillMaxWidth()) {
            Text("Settings", color = Color.White)
        }

        Text(
            "Bridge: $bridgeStatus",
            color = Color.White.copy(alpha = 0.3f),
            fontSize = 11.sp,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.weight(1f))
    }
}
