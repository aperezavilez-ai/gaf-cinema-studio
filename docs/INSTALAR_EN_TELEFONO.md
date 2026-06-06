# Instalar GAF Cinema Studio en tu teléfono Android

## 1. En el teléfono
1. **Ajustes → Acerca del teléfono** → toca **Número de compilación** 7 veces
2. **Opciones de desarrollador → Depuración USB** ON
3. Conecta USB al PC → **Confiar en este equipo**

## 2. En Android Studio
1. Abre carpeta `android/`
2. Arriba elige **tu teléfono** (no el emulador)
3. Pulsa **Run ▶**

## 3. En la app
- Icono: **GAF** dorado en negro
- Nombre: **GAF Cinema Studio**
- **New Project** → **Open editor →**

## Script alternativo (tras un Run exitoso)
```powershell
cd "d:\PROGRAMAS IA\GAF CINEMA STUDIO"
.\scripts\install_android_apk.ps1
```

## Emulador
Si usas emulador y sale *System UI isn't responding* → usa **teléfono USB** o crea emulador **API 34** (más ligero que API 37).
