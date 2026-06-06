package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cinemastudio.engine.AiSuggestionItem

@Composable
fun GuidancePanel(
    suggestions: List<AiSuggestionItem>,
    onExecute: (String) -> Unit,
    onDismiss: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp)
            .background(Color.White.copy(0.04f), RoundedCornerShape(8.dp))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Default.AutoAwesome, contentDescription = null, tint = Color.White.copy(0.6f), modifier = Modifier.size(16.dp))
            Spacer(Modifier.width(8.dp))
            Text(
                "GUÍA CINEMATOGRÁFICA",
                color = Color.White.copy(0.5f),
                fontSize = 11.sp,
                letterSpacing = 2.sp,
            )
        }

        if (suggestions.isEmpty()) {
            Text("Sin sugerencias por ahora", color = Color.White.copy(0.35f), fontSize = 13.sp)
        } else {
            suggestions.forEach { item ->
                SuggestionCard(item, onExecute, onDismiss)
            }
        }
    }
}

@Composable
private fun SuggestionCard(
    item: AiSuggestionItem,
    onExecute: (String) -> Unit,
    onDismiss: (String) -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .background(Color.White.copy(0.06f), RoundedCornerShape(6.dp))
            .padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(item.message, color = Color.White.copy(0.85f), fontSize = 13.sp)
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (item.isActionable) {
                Button(
                    onClick = { onExecute(item.id) },
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black),
                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp),
                    modifier = Modifier.height(32.dp),
                ) {
                    Text(item.actionLabel ?: "Ejecutar", fontSize = 12.sp)
                }
            }
            Spacer(Modifier.weight(1f))
            IconButton(onClick = { onDismiss(item.id) }, modifier = Modifier.size(28.dp)) {
                Icon(Icons.Default.Close, contentDescription = "Dismiss", tint = Color.White.copy(0.4f), modifier = Modifier.size(14.dp))
            }
        }
    }
}
