package com.cinemastudio

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.cinemastudio.engine.EngineBridge

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        EngineBridge.initialize()
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                HomeScreen()
            }
        }
    }
}

@Composable
fun HomeScreen() {
    var status by remember { mutableStateOf(EngineBridge.bridgeStatus()) }

    Column(
        Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text("CINEMASTUDIO", style = MaterialTheme.typography.headlineMedium)
        Text("Android shell — wire UniFFI at integration", style = MaterialTheme.typography.bodyMedium)
        Text("Bridge: $status", style = MaterialTheme.typography.bodySmall)
        Button(onClick = { status = EngineBridge.bridgeStatus() }) {
            Text("Refresh status")
        }
    }
}
