#!/usr/bin/env bash
# macOS 一鍵打包：tauri build → 注入多外觀圖示 → 以注入後的 .app 重做 dmg。
#
# `tauri build` 產出的 dmg 內含「未注入圖示」的 .app，因此 dmg 必須在注入之後重做。
# 完成後會移除 target/ 內的中間 .app，避免 Finder／Spotlight 出現兩個 SessionHub.app。
#
# 用法：
#   scripts/macos-package.sh            # 只產生 dmg
#   scripts/macos-package.sh --install  # 另外覆蓋安裝到 /Applications
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUNDLE="$ROOT/src-tauri/target/release/bundle"
APP="$BUNDLE/macos/SessionHub.app"
INSTALL=0
[[ "${1:-}" == "--install" ]] && INSTALL=1

if ! xcrun --find actool >/dev/null 2>&1; then
  export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
fi

cd "$ROOT"
echo "==> tauri build（僅 .app）"
bun run tauri build --bundles app

"$ROOT/scripts/macos-inject-icon.sh" "$APP"

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
ARCH="$(uname -m)"; [[ "$ARCH" == "arm64" ]] && ARCH="aarch64"
DMG_DIR="$BUNDLE/dmg"
DMG="$DMG_DIR/SessionHub_${VERSION}_${ARCH}.dmg"

echo "==> 重做 dmg：$DMG"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
ditto "$APP" "$STAGE/SessionHub.app"
ln -s /Applications "$STAGE/Applications"
mkdir -p "$DMG_DIR"
rm -f "$DMG"
hdiutil create -volname "SessionHub" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null

if [[ $INSTALL -eq 1 ]]; then
  echo "==> 安裝到 /Applications"
  osascript -e 'tell application "SessionHub" to quit' >/dev/null 2>&1 || true
  rm -rf /Applications/SessionHub.app
  ditto "$APP" /Applications/SessionHub.app
fi

# 中間產物已在 dmg 內（必要時已安裝），移除以免出現第二個 App
rm -rf "$APP"
echo "完成：$DMG"
