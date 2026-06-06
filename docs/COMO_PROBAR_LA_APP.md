# Cómo probar CinemaStudio — A → B → C

## A) Emulador + Run (Android Studio)

1. Abre **solo** la carpeta `android/` en Android Studio.
2. Espera **Gradle Sync** (barra azul desaparece).
3. **Tools → Device Manager** → Pixel 7 Pro → **▶ Play** (una vez).
4. Espera el **escritorio Android** (no pantalla negra).
5. Arriba: **app** + **Pixel 7 Pro** → **Run ▶**.
6. En la app: **New Project** → **Open editor →**.

Si dice *"already running"*: pulsa OK y **Run ▶** (no vuelvas a pulsar Play).

---

## B) Solo teléfono Android (sin emulador)

1. En el móvil: **Ajustes → Acerca del teléfono** → toca **Número de compilación** 7 veces.
2. **Opciones de desarrollador → Depuración USB** ON.
3. Conecta USB al PC → acepta **Confiar en este equipo**.
4. En Android Studio: abre `android/` → elige tu teléfono arriba → **Run ▶** (genera el APK).
5. O desde PowerShell (tras un Run exitoso):

```powershell
.\scripts\install_android_apk.ps1
```

---

## C) Errores comunes

| Error | Solución |
|-------|----------|
| Gradle sync failed | **File → Sync Project with Gradle Files**; revisa internet |
| JDK not found | **File → Settings → Build → Gradle → JDK 17** |
| Emulator disconnected | Device Manager → **Stop** → 15 s → **▶ Play** |
| already running | OK → busca ventana emulador → **Run ▶** |
| INSTALL_FAILED | Desinstala CinemaStudio vieja del teléfono e intenta otra vez |
| Build failed (rojo) | Pestaña **Build** → copia el error → pégalo en Cursor |
| Kotlin compile daemon failed | **File → Sync**; ya aplicado `kotlin.compiler.execution.strategy=in-process` en `gradle.properties` |

---

La app **ya está construida** en el repo. A/B solo la **instalan** para verla en pantalla.
