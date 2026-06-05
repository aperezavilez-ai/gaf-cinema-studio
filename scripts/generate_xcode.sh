#!/usr/bin/env bash
# Generate Xcode project from project.yml (requires xcodegen on Mac)
set -euo pipefail
cd "$(dirname "$0")/../ios"
if ! command -v xcodegen &>/dev/null; then
  echo "Install: brew install xcodegen"
  exit 1
fi
xcodegen generate
echo "Open: open CinemaStudio.xcodeproj"
