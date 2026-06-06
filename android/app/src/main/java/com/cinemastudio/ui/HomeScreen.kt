package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.cinemastudio.engine.EngineBridge
import com.cinemastudio.engine.ProjectSession

@Composable
fun HomeScreen(
    onOpenEditor: () -> Unit,
    onOpenSettings: () -> Unit,
    vm: HomeViewModel = viewModel(),
) {
    val context = LocalContext.current

    LaunchedEffect(Unit) {
        vm.projectName = ProjectSession.projectName
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        Spacer(modifier = Modifier.height(48.dp))

        GafBrandHeader()

        Spacer(modifier = Modifier.height(8.dp))

        vm.projectName?.let { name ->
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
                vm.createProject(context, "Demo Film") { onOpenEditor() }
            },
            modifier = Modifier.fillMaxWidth(),
            colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
        ) {
            Text("New Project")
        }

        OutlinedButton(onClick = onOpenSettings, modifier = Modifier.fillMaxWidth()) {
            Text("Settings", color = Color.White)
        }

        vm.errorMessage?.let { msg ->
            Text(msg, color = Color(0xFFFF6B6B), fontSize = 12.sp, textAlign = TextAlign.Center)
        }

        Text(
            "Engine: ${if (EngineBridge.useNativeEngine) "native" else "mock (local editor)"}",
            color = Color.White.copy(alpha = 0.3f),
            fontSize = 11.sp,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.weight(1f))
    }
}
