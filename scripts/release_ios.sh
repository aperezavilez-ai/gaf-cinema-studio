#!/usr/bin/env bash
# TestFlight release scaffold — run on Mac with Apple Developer account.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IOS="$ROOT/ios"
SCHEME="CinemaStudio"
ARCHIVE="$IOS/build/CinemaStudio.xcarchive"
EXPORT="$IOS/build/export"

echo "==> CinemaStudio iOS Release (TestFlight scaffold)"

cd "$IOS"
if [ ! -d CinemaStudio.xcodeproj ]; then
  command -v xcodegen >/dev/null || { echo "Install: brew install xcodegen"; exit 1; }
  xcodegen generate
fi

echo "==> Archive (requires signing team in ios/project.yml DEVELOPMENT_TEAM)"
xcodebuild \
  -scheme "$SCHEME" \
  -configuration Release \
  -archivePath "$ARCHIVE" \
  archive \
  CODE_SIGN_STYLE=Automatic

echo "==> Export IPA for TestFlight"
xcodebuild \
  -exportArchive \
  -archivePath "$ARCHIVE" \
  -exportPath "$EXPORT" \
  -exportOptionsPlist "$ROOT/release/ios/ExportOptions.plist"

echo "==> Upload with Fastlane (optional):"
echo "  cd fastlane && fastlane beta"
echo "IPA: $EXPORT/CinemaStudio.ipa"
