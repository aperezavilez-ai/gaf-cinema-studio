package com.cinemastudio.engine

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.ColorMatrix
import android.graphics.ColorMatrixColorFilter
import android.graphics.Paint

object ColorGradeUtil {
    fun apply(bitmap: Bitmap, brightness: Float, contrast: Float, saturation: Float): Bitmap {
        if (brightness == 0f && contrast == 1f && saturation == 1f) return bitmap
        val out = Bitmap.createBitmap(bitmap.width, bitmap.height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(out)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG)

        val cm = ColorMatrix()
        if (saturation != 1f) cm.setSaturation(saturation)
        if (contrast != 1f) {
            val scale = contrast
            val translate = (1f - scale) * 128f + brightness * 128f
            val contrastMatrix = ColorMatrix(
                floatArrayOf(
                    scale, 0f, 0f, 0f, translate,
                    0f, scale, 0f, 0f, translate,
                    0f, 0f, scale, 0f, translate,
                    0f, 0f, 0f, 1f, 0f,
                )
            )
            cm.postConcat(contrastMatrix)
        } else if (brightness != 0f) {
            val translate = brightness * 128f
            val brightMatrix = ColorMatrix(
                floatArrayOf(
                    1f, 0f, 0f, 0f, translate,
                    0f, 1f, 0f, 0f, translate,
                    0f, 0f, 1f, 0f, translate,
                    0f, 0f, 0f, 1f, 0f,
                )
            )
            cm.postConcat(brightMatrix)
        }

        paint.colorFilter = ColorMatrixColorFilter(cm)
        canvas.drawBitmap(bitmap, 0f, 0f, paint)
        return out
    }
}
