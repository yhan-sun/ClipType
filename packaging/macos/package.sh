#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.2.0-beta.1-dev}"
UNIVERSAL_BINARY="${2:?universal binary path required}"
OUTPUT_DIR="${3:-dist/macos}"
APP="$OUTPUT_DIR/ClipType.app"
CONTENTS="$APP/Contents"
RESOURCES="$CONTENTS/Resources"

rm -rf "$OUTPUT_DIR"
mkdir -p "$CONTENTS/MacOS" "$RESOURCES"
install -m 0755 "$UNIVERSAL_BINARY" "$CONTENTS/MacOS/ClipType"

SHORT_VERSION="${VERSION#v}"
SHORT_VERSION="${SHORT_VERSION%%-*}"
BUILD_VERSION="${GITHUB_RUN_NUMBER:-1}"
sed -e "s/__SHORT_VERSION__/$SHORT_VERSION/g" \
    -e "s/__BUILD_VERSION__/$BUILD_VERSION/g" \
    packaging/macos/Info.plist.template > "$CONTENTS/Info.plist"

render_svg() {
  local source="$1" destination="$2"
  if sips -s format png "$source" --out "$destination" >/dev/null 2>&1; then
    return 0
  fi
  local temp
  temp="$(mktemp -d)"
  qlmanage -t -s 1024 -o "$temp" "$source" >/dev/null 2>&1
  local rendered
  rendered="$(find "$temp" -type f -name '*.png' -print -quit)"
  test -n "$rendered"
  cp "$rendered" "$destination"
  rm -rf "$temp"
}

SOURCE_PNG="$OUTPUT_DIR/cliptype-primary.png"
render_svg assets/branding/cliptype-primary.svg "$SOURCE_PNG"
ICONSET="$OUTPUT_DIR/ClipType.iconset"
mkdir -p "$ICONSET"
for spec in \
  '16 icon_16x16.png' '32 icon_16x16@2x.png' \
  '32 icon_32x32.png' '64 icon_32x32@2x.png' \
  '128 icon_128x128.png' '256 icon_128x128@2x.png' \
  '256 icon_256x256.png' '512 icon_256x256@2x.png' \
  '512 icon_512x512.png' '1024 icon_512x512@2x.png'; do
  size="${spec%% *}"
  name="${spec#* }"
  sips -z "$size" "$size" "$SOURCE_PNG" --out "$ICONSET/$name" >/dev/null
 done
iconutil -c icns "$ICONSET" -o "$RESOURCES/ClipType.icns"
cp assets/branding/cliptype-status-template.svg "$RESOURCES/ClipTypeStatusTemplate.svg"
cp LICENSE-MIT LICENSE-APACHE "$RESOURCES/"
cp docs/CONFIGURATION.md "$RESOURCES/CONFIGURATION.md"

plutil -lint "$CONTENTS/Info.plist"
lipo -archs "$CONTENTS/MacOS/ClipType" | grep -q arm64
lipo -archs "$CONTENTS/MacOS/ClipType" | grep -q x86_64
codesign --force --deep --sign - "$APP"
codesign --verify --deep --strict "$APP"

mkdir -p "$OUTPUT_DIR/image"
cp -R "$APP" "$OUTPUT_DIR/image/"
ln -s /Applications "$OUTPUT_DIR/image/Applications"
hdiutil create -volname ClipType -srcfolder "$OUTPUT_DIR/image" -ov -format UDZO "$OUTPUT_DIR/ClipType-$VERSION-macos-universal.dmg" >/dev/null
ditto -c -k --sequesterRsrc --keepParent "$APP" "$OUTPUT_DIR/ClipType-$VERSION-macos-universal.zip"
(
  cd "$OUTPUT_DIR"
  shasum -a 256 "ClipType-$VERSION-macos-universal.dmg" "ClipType-$VERSION-macos-universal.zip" > SHA256SUMS.txt
  shasum -a 256 -c SHA256SUMS.txt
)
rm -rf "$OUTPUT_DIR/image" "$ICONSET" "$SOURCE_PNG"
