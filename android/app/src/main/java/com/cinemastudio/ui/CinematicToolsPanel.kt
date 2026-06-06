package com.cinemastudio.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CameraAlt
import androidx.compose.material.icons.filled.Lens
import androidx.compose.material.icons.filled.Palette
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cinemastudio.engine.TimelineClip

private val GafGold = Color(0xFFC9A227)

@Composable
fun CinematicToolsPanel(
    selectedClip: TimelineClip?,
    activeCameraAngle: Int,
    onLensChange: (String) -> Unit,
    onColorChange: (brightness: Float, contrast: Float, saturation: Float) -> Unit,
    onCameraAngleChange: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (selectedClip == null) return

    var brightness by remember(selectedClip.id) { mutableFloatStateOf(selectedClip.brightness) }
    var contrast by remember(selectedClip.id) { mutableFloatStateOf(selectedClip.contrast) }
    var saturation by remember(selectedClip.id) { mutableFloatStateOf(selectedClip.saturation) }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp)
            .background(Color.White.copy(0.04f), RoundedCornerShape(8.dp))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Text("HERRAMIENTAS CINEMA", color = Color.White.copy(0.5f), fontSize = 11.sp, letterSpacing = 2.sp)

        // Lenses
        Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
            Icon(Icons.Default.Lens, null, tint = Color.White.copy(0.5f), modifier = Modifier.size(16.dp))
            Spacer(Modifier.width(8.dp))
            Text("Lentes", color = Color.White.copy(0.7f), fontSize = 13.sp)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            listOf("none" to "Normal", "anamorphic" to "Anamórfico", "vintage" to "Vintage", "wide" to "Wide").forEach { (id, label) ->
                LensChip(label, selected = selectedClip.lensPreset == id) { onLensChange(id) }
            }
        }

        // Color
        Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
            Icon(Icons.Default.Palette, null, tint = Color.White.copy(0.5f), modifier = Modifier.size(16.dp))
            Spacer(Modifier.width(8.dp))
            Text("Colorimetría", color = Color.White.copy(0.7f), fontSize = 13.sp)
        }
        ColorSlider("Brillo", brightness, -1f, 1f) {
            brightness = it
            onColorChange(brightness, contrast, saturation)
        }
        ColorSlider("Contraste", contrast, 0.5f, 1.5f) {
            contrast = it
            onColorChange(brightness, contrast, saturation)
        }
        ColorSlider("Saturación", saturation, 0f, 2f) {
            saturation = it
            onColorChange(brightness, contrast, saturation)
        }

        // Multicam
        Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
            Icon(Icons.Default.CameraAlt, null, tint = Color.White.copy(0.5f), modifier = Modifier.size(16.dp))
            Spacer(Modifier.width(8.dp))
            Text("Multicámara", color = Color.White.copy(0.7f), fontSize = 13.sp)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            (1..3).forEach { angle ->
                FilterChip(
                    selected = activeCameraAngle == angle,
                    onClick = { onCameraAngleChange(angle) },
                    label = { Text("Ángulo $angle", fontSize = 12.sp) },
                    colors = FilterChipDefaults.filterChipColors(
                        selectedContainerColor = GafGold.copy(0.35f),
                        selectedLabelColor = Color.White,
                        labelColor = Color.White.copy(0.6f),
                    ),
                )
            }
        }
        Text(
            "Asigna ángulo al clip seleccionado · Cambia ángulo para alternar tomas",
            color = Color.White.copy(0.3f),
            fontSize = 10.sp,
        )
    }
}

@Composable
private fun LensChip(label: String, selected: Boolean, onClick: () -> Unit) {
    val bg = if (selected) GafGold.copy(0.35f) else Color.White.copy(0.08f)
    val border = if (selected) GafGold else Color.White.copy(0.15f)
    TextButton(
        onClick = onClick,
        modifier = Modifier
            .border(1.dp, border, RoundedCornerShape(4.dp))
            .background(bg, RoundedCornerShape(4.dp))
            .height(32.dp),
        contentPadding = PaddingValues(horizontal = 10.dp),
    ) {
        Text(label, color = Color.White.copy(if (selected) 1f else 0.65f), fontSize = 11.sp)
    }
}

@Composable
private fun ColorSlider(label: String, value: Float, range: ClosedFloatingPointRange<Float>, onChange: (Float) -> Unit) {
    Column {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, color = Color.White.copy(0.55f), fontSize = 11.sp)
            Text("%.2f".format(value), color = Color.White.copy(0.4f), fontSize = 10.sp)
        }
        Slider(
            value = value,
            onValueChange = onChange,
            valueRange = range,
            colors = SliderDefaults.colors(thumbColor = GafGold, activeTrackColor = GafGold.copy(0.7f)),
        )
    }
}
