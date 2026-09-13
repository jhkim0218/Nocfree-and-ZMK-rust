#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
swift test
swift build -c release --arch arm64 --arch x86_64
binary_dir="$(swift build -c release --arch arm64 --arch x86_64 --show-bin-path)"
bundle="dist/NocFree Battery.app"
mkdir -p "$bundle/Contents/MacOS"
cp "$binary_dir/NocFreeBattery" "$bundle/Contents/MacOS/NocFreeBattery"
cp Info.plist "$bundle/Contents/Info.plist"
plutil -lint "$bundle/Contents/Info.plist"
lipo -verify_arch arm64 x86_64 "$bundle/Contents/MacOS/NocFreeBattery"
if [[ -n "${SIGN_IDENTITY:-}" ]]; then
    codesign --force --sign "$SIGN_IDENTITY" --options runtime --timestamp "$bundle"
else
    codesign --force --sign - --options runtime --timestamp=none "$bundle"
fi
codesign --verify --strict "$bundle"
shasum -a 256 "$bundle/Contents/MacOS/NocFreeBattery"
echo "App bundle built: $bundle"
