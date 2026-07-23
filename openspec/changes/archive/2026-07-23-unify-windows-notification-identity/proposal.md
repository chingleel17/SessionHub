## Why

SessionHub 目前透過 Tauri 與 provider hook 的 `snoretoast.exe` 兩條路徑發送 Windows 通知。兩者未共用相同的 Windows 應用程式身分，導致通知中心將它們歸類到 SessionHub、終端機或其他發送程序，並顯示不一致的圖示，降低辨識性。

## What Changes

- 為所有由 SessionHub 發送的 Windows Toast 建立並使用一致的應用程式身分。
- 讓 hook 離線通知使用 SessionHub 的開始功能表捷徑與應用程式圖示，而非執行 hook 的終端機身分。
- 保留 hook 在 SessionHub 未運行時仍可通知、通知設定開關與同 session 去重的既有行為。
- 讓應用程式內通知與 hook 通知在 Windows 通知中心統一歸類為 SessionHub。

## Capabilities

### New Capabilities

- `windows-notification-identity`: 定義 SessionHub 在 Windows 的統一 Toast 應用程式身分、圖示與離線 hook 通知歸類行為。

### Modified Capabilities

- `hook-native-notification`: 變更 hook 離線通知的 Windows Toast 歸屬與圖示要求。
- `intervention-notification`: 變更應用程式內與 hook 通知的一致歸類要求。

## Impact

- 影響 `src-tauri` 的 Windows 安裝包、應用程式初始化與通知設定。
- 影響 Claude、Codex、Copilot hook 的共用 `notify.cjs` 與隨附 `snoretoast.exe` 呼叫參數。
- 可能需要新增或調整 Windows 安裝捷徑與應用程式識別相關的打包設定；不變更前端 IPC 或使用者設定欄位。
