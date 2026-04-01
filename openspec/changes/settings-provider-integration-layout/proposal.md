## Why

目前設定頁把 `Copilot` / `OpenCode` 的 provider integration 管理內容塞在一般設定卡片內，導致桌面寬畫面右側大量留白，但最需要閱讀與操作的區塊反而被擠在窄欄中。隨著 bridge/plugin 管理功能增加，現有排版已影響可讀性、掃描效率與操作舒適度。

## What Changes

- 調整設定頁資訊架構，將一般設定與 provider integration 管理拆成獨立區塊，而不是同擠在單一卡片中。
- 定義桌面寬畫面的寬版排版需求，讓 provider integration 區塊優先使用主要內容寬度。
- 明確規範 provider integration 卡片內的資訊區與操作區排版，避免路徑、狀態與按鈕因欄寬不足而擁擠換行。
- 保留窄畫面與較小視窗下的可回退堆疊式布局，確保設定頁仍具備響應式表現。

## Capabilities

### New Capabilities

### Modified Capabilities
- `app-settings`: 調整設定頁在 provider integration 管理區塊的資訊架構與響應式排版要求。

## Impact

- `src/components/SettingsView.tsx`
- `src/App.css`
- `openspec/specs/app-settings/spec.md`
