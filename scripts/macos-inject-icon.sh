#!/usr/bin/env bash
# 將 Icon Composer 的 .icon 編譯為 Assets.car 並注入打包後的 macOS .app，使 Dock／Launchpad 圖示
# 能隨 macOS 26 的圖示風格（預設、深色、clear／tinted）切換。
#
# Tauri bundler 目前只放入 icon.icns，不處理 .icon，因此在 `tauri build` 之後執行本腳本。
#
# 用法：
#   scripts/macos-inject-icon.sh [path/to/SessionHub.app]
#
# 環境變數：
#   CODESIGN_IDENTITY  重簽使用的身分；預設 "-"（ad-hoc）
#   ICON_NAME          .icon 資源名稱（不含副檔名）；預設 AppIcon
#   DEVELOPER_DIR      未以 xcode-select 切到 Xcode 時，預設指向 /Applications/Xcode.app
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="${1:-$ROOT/src-tauri/target/release/bundle/macos/SessionHub.app}"
ICON_NAME="${ICON_NAME:-AppIcon}"
ICON_SRC="$ROOT/src-tauri/icons/macos/$ICON_NAME.icon"
IDENTITY="${CODESIGN_IDENTITY:--}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "僅支援 macOS" >&2; exit 1
fi
if [[ ! -d "$APP" ]]; then
  echo "找不到 .app：$APP（請先執行 bun run tauri build --bundles app）" >&2; exit 1
fi
if [[ ! -d "$ICON_SRC" ]]; then
  echo "找不到 .icon 資源：$ICON_SRC" >&2; exit 1
fi

# actool 需要完整 Xcode；只有 Command Line Tools 時改指向 Xcode.app
if ! xcrun --find actool >/dev/null 2>&1; then
  export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
  xcrun --find actool >/dev/null 2>&1 || { echo "找不到 actool：請安裝 Xcode" >&2; exit 1; }
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "==> 編譯 $ICON_NAME.icon"
xcrun actool "$ICON_SRC" \
  --compile "$WORK" \
  --output-format human-readable-text \
  --notices --warnings --errors \
  --output-partial-info-plist "$WORK/partial.plist" \
  --app-icon "$ICON_NAME" \
  --include-all-app-icons \
  --enable-on-demand-resources NO \
  --development-region en \
  --target-device mac \
  --minimum-deployment-target 26.0 \
  --platform macosx

[[ -f "$WORK/Assets.car" ]] || { echo "actool 未產生 Assets.car" >&2; exit 1; }

RES="$APP/Contents/Resources"
PLIST="$APP/Contents/Info.plist"

echo "==> 注入 Assets.car 與 Info.plist"
cp "$WORK/Assets.car" "$RES/Assets.car"
# icon.icns 保留給不支援多外觀的舊系統
if /usr/libexec/PlistBuddy -c "Print :CFBundleIconName" "$PLIST" >/dev/null 2>&1; then
  /usr/libexec/PlistBuddy -c "Set :CFBundleIconName $ICON_NAME" "$PLIST"
else
  /usr/libexec/PlistBuddy -c "Add :CFBundleIconName string $ICON_NAME" "$PLIST"
fi

echo "==> 重簽（identity: ${IDENTITY}）"
codesign --force --deep --sign "$IDENTITY" "$APP"
codesign --verify --deep --strict "$APP"

# 讓 LaunchServices 與 Dock 重新讀取圖示
touch "$APP"
echo "完成：$APP"
echo "提示：若 Dock 圖示未更新，執行 killall Dock；圖示風格只在打包後的 .app 生效。"
