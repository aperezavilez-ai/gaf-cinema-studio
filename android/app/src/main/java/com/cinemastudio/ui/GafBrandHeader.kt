package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private val GafGold = Color(0xFFC9A227)

@Composable
fun GafBrandHeader(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = "GAF",
            color = GafGold,
            fontSize = 40.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 10.sp,
        )
        Spacer(modifier = Modifier.height(10.dp))
        Spacer(
            modifier = Modifier
                .width(56.dp)
                .height(1.dp)
                .background(GafGold.copy(alpha = 0.55f))
        )
        Spacer(modifier = Modifier.height(14.dp))
        Text(
            text = "CINEMA",
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Light,
            letterSpacing = 6.sp,
        )
        Text(
            text = "STUDIO",
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Light,
            letterSpacing = 6.sp,
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "Mobile cinematic studio",
            color = Color.White.copy(alpha = 0.45f),
            fontSize = 13.sp,
            textAlign = TextAlign.Center,
            style = MaterialTheme.typography.bodySmall,
        )
    }
}
