package com.cinemastudio.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val CinemaDark = darkColorScheme(
    primary = Color.White,
    onPrimary = Color.Black,
    background = Color.Black,
    surface = Color(0xFF141414),
    onBackground = Color.White,
    onSurface = Color.White
)

@Composable
fun CinemaStudioTheme(content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = CinemaDark, content = content)
}
