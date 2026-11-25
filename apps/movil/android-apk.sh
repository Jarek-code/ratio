#!/usr/bin/env bash
# Genera el APK de Ratio para un teléfono Android (arm64-v8a) y lo deja en una ruta fija.
#
# Requisitos, todos gratuitos:
#   - Android SDK con el NDK instalado (sdkmanager "ndk;30.0.16248370")
#   - Los objetivos de Rust para Android: rustup target add aarch64-linux-android
#   - JDK 17 y el CLI de Dioxus (`dx`)
#
# Uso:
#   ./android-apk.sh              compila y deja target/ratio-android.apk
#   ./android-apk.sh --instalar   además lo instala con adb en el teléfono conectado
#
# Variables que se pueden ajustar: ANDROID_HOME, ANDROID_NDK_HOME, JAVA_HOME.

set -euo pipefail

AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIZ="$(cd "$AQUI/../.." && pwd)"
ABI="aarch64-linux-android"
SALIDA="$RAIZ/target/ratio-android.apk"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    ANDROID_NDK_HOME="$(ls -d "$ANDROID_HOME"/ndk/* 2>/dev/null | sort -V | tail -1)"
fi
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-}"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-17-openjdk-amd64}"
# Gradle escribe sus cachés fuera del proyecto: se dejan dentro de target/
export GRADLE_USER_HOME="${GRADLE_USER_HOME:-$RAIZ/target/gradle-home}"
export ANDROID_USER_HOME="${ANDROID_USER_HOME:-$RAIZ/target/android-home}"
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$ANDROID_HOME/platform-tools:$PATH"

if [[ ! -d "$ANDROID_NDK_HOME" ]]; then
    echo "No se encontró el NDK en $ANDROID_NDK_HOME" >&2
    echo "Instálalo con: sdkmanager \"ndk;30.0.16248370\"" >&2
    exit 1
fi

# Los binarios de otras arquitecturas que hayan quedado de compilaciones anteriores
# (por ejemplo x86_64 del emulador) se retiran para que el APK quede solo con arm64.
find "$RAIZ/target/dx" -path "*jniLibs*" -mindepth 1 -maxdepth 1 -type d \
    ! -name "arm64-v8a" -exec rm -rf {} + 2>/dev/null || true

echo "Compilando para $ABI…"
cd "$AQUI"
dx build --platform android --release --target "$ABI"

APK="$(find "$RAIZ/target/dx" -name '*.apk' -path "*release*" | head -1)"
if [[ -z "$APK" ]]; then
    echo "dx terminó sin producir un APK" >&2
    exit 1
fi

cp "$APK" "$SALIDA"
echo
echo "APK: $SALIDA ($(du -h "$SALIDA" | cut -f1))"
echo "Arquitectura: $(unzip -l "$SALIDA" | awk '/lib\/.*\.so/ {print $4}' | cut -d/ -f2 | sort -u | tr '\n' ' ')"

if [[ "${1:-}" == "--instalar" ]]; then
    echo
    echo "Instalando en el teléfono conectado…"
    adb install -r "$SALIDA"
    adb shell am start -n com.ratio.app/dev.dioxus.main.MainActivity
fi
