#!/usr/bin/env bash
set -e
# Usage: ./create-macos-dmg.sh <artifact_name> <target_triple>
# Example: ./create-macos-dmg.sh psweep-macos-aarch64 aarch64-apple-darwin
# DMG contains a single "Port Sweeper.app". When run from the DMG, the app installs itself and configures CLI (see src/installer_macos.rs).
ARTIFACT_NAME="$1"
TARGET="$2"

# Use target/triple/release when present (CI); else target/release (local "cargo build --release")
RELEASE_DIR="target/${TARGET}/release"
if [ ! -f "${RELEASE_DIR}/psweep" ]; then
  RELEASE_DIR="target/release"
fi
if [ ! -f "${RELEASE_DIR}/psweep" ]; then
  echo "Error: ${RELEASE_DIR}/psweep not found. Run: cargo build --release [--target ${TARGET}]"
  exit 1
fi

APP_NAME="Port Sweeper"
APP_BUNDLE="${APP_NAME}.app"
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

cp "${RELEASE_DIR}/psweep" "${APP_BUNDLE}/Contents/MacOS/"
chmod +x "${APP_BUNDLE}/Contents/MacOS/psweep"

if [ -f "assets/AppIcon.icns" ]; then
  cp "assets/AppIcon.icns" "${APP_BUNDLE}/Contents/Resources/AppIcon.icns"
fi

cat > "${APP_BUNDLE}/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>psweep</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>com.dillibabukadati.port-sweeper</string>
  <key>CFBundleName</key>
  <string>Port Sweeper</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
</dict>
</plist>
PLIST

DMG_DIR="dmg"
mkdir -p "${DMG_DIR}"
cp -R "${APP_BUNDLE}" "${DMG_DIR}/"
ln -sf /Applications "${DMG_DIR}/Applications"

hdiutil create -volname "Port Sweeper" -srcfolder "${DMG_DIR}" -ov -format UDZO -o "${ARTIFACT_NAME}.dmg"
