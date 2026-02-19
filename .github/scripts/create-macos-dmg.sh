#!/usr/bin/env bash
set -e
# Usage: ./create-macos-dmg.sh <artifact_name> <target_triple>
# Example: ./create-macos-dmg.sh psweep-macos-aarch64 aarch64-apple-darwin
ARTIFACT_NAME="$1"
TARGET="$2"

APP_NAME="Port Sweeper"
APP_BUNDLE="${APP_NAME}.app"
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
cp "target/${TARGET}/release/port-sweeper" "${APP_BUNDLE}/Contents/MacOS/"
cp "target/${TARGET}/release/psweep" "${APP_BUNDLE}/Contents/MacOS/"
chmod +x "${APP_BUNDLE}/Contents/MacOS/port-sweeper" "${APP_BUNDLE}/Contents/MacOS/psweep"

cat > "${APP_BUNDLE}/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>port-sweeper</string>
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

# Single installer: copies app to Applications and adds CLI to PATH (one action, one password prompt)
cat > "${DMG_DIR}/Install Port Sweeper.command" << 'SCRIPT'
#!/bin/bash
set -e
APP_NAME="Port Sweeper"
DMG_APP="$(dirname "$0")/${APP_NAME}.app"
DEST="/Applications/${APP_NAME}.app"

echo "Port Sweeper Installer"
echo "----------------------"
echo "This will install the app to Applications and add 'psweep' and 'port-sweeper' to your PATH."
echo "You may be asked for your password once."
echo ""

if [ ! -d "$DMG_APP" ]; then
  echo "Error: ${APP_NAME}.app not found next to this script."
  read -p "Press Enter to close."
  exit 1
fi

echo "Installing ${APP_NAME}.app to /Applications..."
sudo cp -R "$DMG_APP" /Applications/

echo "Adding psweep and port-sweeper to /usr/local/bin..."
sudo mkdir -p /usr/local/bin
sudo ln -sf "${DEST}/Contents/MacOS/psweep" /usr/local/bin/psweep
sudo ln -sf "${DEST}/Contents/MacOS/port-sweeper" /usr/local/bin/port-sweeper

echo ""
echo "Done! You can open Port Sweeper from Applications or Spotlight,"
echo "and use 'psweep' or 'port-sweeper' in the terminal."
read -p "Press Enter to close."
SCRIPT
chmod +x "${DMG_DIR}/Install Port Sweeper.command"

hdiutil create -volname "Port Sweeper" -srcfolder "${DMG_DIR}" -ov -format UDZO -o "${ARTIFACT_NAME}.dmg"
