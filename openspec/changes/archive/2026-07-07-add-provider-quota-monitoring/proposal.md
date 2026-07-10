## Why

SessionHub 目前能管理 session、bridge 與 analytics，但還無法直接顯示各平台剩餘額度、已使用量與 reset 時間，使用者仍需要切回 CLI、外部插件或各平台後台確認。你希望參考 `opencode-quota` 的方向，把這類計量能力直接整合進 SessionHub，並保留未來以插件式 connector 擴充 quota provider 的空間。

## What Changes

- 新增 provider quota monitoring 能力，讓 SessionHub 可查詢並顯示各平台的 quota / usage / reset 資訊。
- 在後端建立 quota adapter / connector 架構，優先支援內建實作，並保留可選的插件式 provider connector 擴充方式。
- 新增 quota 快取與 refresh 流程，支援應用啟動、背景輪詢、手動刷新，以及 bridge 事件後的節流更新。
- 擴充 Dashboard、全域狀態列與設定頁，讓使用者能在 SessionHub 內查看 provider quota 狀態與資料來源。

## Capabilities

### New Capabilities
- `provider-quota-monitoring`: 定義 quota snapshot、provider adapter / connector、更新策略與 UI 顯示需求。

### Modified Capabilities
- `app-settings`: 新增 quota 相關設定，例如是否啟用 quota 監控、刷新間隔與顯示偏好。
- `global-status-bar`: 擴充狀態列，使其可顯示簡化的 provider quota 摘要。
- `dashboard-analytics-panel`: 擴充 Dashboard，加入 provider quota overview 與最近更新資訊。
- `provider-integration`: 補充 provider integration 卡片的 quota source / auth / refresh 診斷資訊與操作入口。

## Impact

- Rust backend: 新增 quota manager、provider adapters/connectors、快取與 refresh command。
- Frontend: `src/App.tsx`、Dashboard、Settings、status bar 需加入 quota query、快取與顯示元件。
- Local data: 可能需要新增 quota snapshot 快取結構，避免每次畫面刷新都直接打遠端來源。
- OpenSpec: 新增 `provider-quota-monitoring` capability，並更新 settings、dashboard、status bar、provider integration 的既有規格。
