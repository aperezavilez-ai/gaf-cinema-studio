package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BetaProgramScreen(onBack: () -> Unit) {
    var completions by remember { mutableIntStateOf(0) }
    val target = 10

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Beta Program", color = Color.White) },
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
            Modifier.fillMaxSize().padding(padding).padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Text("GAF Cinema Studio MVP 1.0.0", color = Color.White, style = MaterialTheme.typography.titleMedium)
            Text("$completions / $target beta projects", color = Color.White.copy(0.7f))
            LinearProgressIndicator(
                progress = { completions.toFloat() / target },
                modifier = Modifier.fillMaxWidth(),
                color = Color.White,
                trackColor = Color.White.copy(0.2f)
            )
            Button(onClick = { if (completions < target) completions++ }) {
                Text("Mark project complete (demo)")
            }
            if (completions >= target) {
                Text("MVP ready to ship", color = Color.Green)
            }
        }
    }
}
