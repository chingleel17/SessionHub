## Why

SessionHub 目前支援 Copilot 與 OpenCode，但尚未支援 Codex 的本機 session，導致使用者無法在同一個介面集中管理第三個常用 AI coding provider。Codex 的 session 檔案採用 `~/.codex/sessions/<YYYY>/<MM>/<DD>/` 的日期分層目錄，若不補上對應掃描能力，使用者必須另外手動進入檔案系統查找與追蹤工作紀錄。

## What Changes

- 新增 Codex provider，讓系統可以掃描 `~/.codex/sessions` 底下依年、月、日分層的 session 檔案。
- 將 Codex session 解析為既有 `SessionInfo` 模型，併入現有 session 清單、專案分組與排序流程。
- 擴充設定頁與 provider 啟用設定，讓使用者可以設定 Codex root 路徑、啟用或停用 Codex 掃描。
- 擴充 provider 相關 UI，讓篩選、標籤與設定頁的 provider 啟用流程能正確處理第三個 provider。

## Capabilities

### New Capabilities

- `codex-provider`: 定義 Codex session 的掃描來源、資料解析、專案分組與與既有 provider 併存的行為。

### Modified Capabilities

- `app-settings`: 新增 Codex root 預設值與 `enabled_providers` / 設定頁對第三個 provider 的要求。
- `provider-filter`: 將 provider 篩選由雙 provider 擴充為支援 Codex。
- `provider-tag`: 定義 Codex session 在卡片上的辨識標籤行為。

## Impact

- Rust backend: `src-tauri/src/lib.rs` 的設定模型、session 掃描流程、快取與檔案監聽邏輯。
- Frontend: `src/App.tsx` 的 provider 設定、session 查詢參數、篩選與顯示元件 props。
- Types and i18n: `src/types/index.ts` 與翻譯字串需加入 Codex provider 值與文案。
- Local settings/data: `settings.json` 需要支援 Codex root 與 provider 啟用狀態；掃描快取與 metadata key 需可容納 Codex session。
