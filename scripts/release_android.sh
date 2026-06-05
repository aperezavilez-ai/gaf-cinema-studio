#!/usr/bin/env bash
# Play Internal Testing release scaffold.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ANDROID="$ROOT/android"

echo "==> CinemaStudio Android Release (Internal Testing scaffold)"

cd "$ANDROID"

if [ ! -f gradlew ]; then
  echo "Generate wrapper: gradle wrapper (requires Gradle installed)"
  exit 1
fi

./gradlew assembleRelease bundleRelease

echo "==> Outputs:"
echo "  APK: app/build/outputs/apk/release/"
echo "  AAB: app/build/outputs/bundle/release/"
echo ""
echo "Upload AAB to Play Console → Testing → Internal testing"
echo "See docs/BETA_RELEASE.md"
