## ADDED Requirements

### Requirement: Antigravity managed hook integration

系統 SHALL 將 Antigravity 納入既有 provider 安裝式整合架構（與 Claude／Codex／Copilot／OpenCode 相同模式），提供安裝、偵測、解除安裝三個操作，由 SessionHub 在全域 `~/.gemini/config/hooks.json` 寫入固定的 marker hook 群組，不提供使用者手動新增／編輯／刪除任意 hook 的介面。

#### Scenario: 安裝 Antigravity 整合

- **WHEN** 使用者在設定中啟用 Antigravity provider 或按下安裝按鈕
- **THEN** 系統於 `~/.gemini/config/hooks.json` 寫入 SessionHub 專屬的 marker hook 群組（群組名含 `sessionhub-provider-event-bridge` 標記），保留檔案中既有的其他群組不受影響

#### Scenario: 偵測整合狀態

- **WHEN** 系統檢查 Antigravity 整合狀態
- **THEN** 依 hooks.json 是否存在、是否含 SessionHub marker 群組，回傳 `installed`／`missing`／`error` 狀態，供前端顯示為與其他 provider 一致的安裝狀態卡片

#### Scenario: 解除安裝

- **WHEN** 使用者按下解除安裝按鈕
- **THEN** 系統自 hooks.json 移除 SessionHub marker 群組，保留其餘使用者或其他工具寫入的群組不受影響

#### Scenario: hooks.json 不存在

- **WHEN** 目標 `~/.gemini/config/hooks.json` 不存在
- **THEN** 系統偵測狀態回傳 `missing`，不回報錯誤；安裝時自動建立檔案與所需目錄

### Requirement: Antigravity hook schema compatibility

系統 SHALL 依 Antigravity hook schema 讀寫 hooks.json，schema 為：頂層以群組名對應物件，物件含可選 `enabled` 與各事件鍵，事件鍵對應 matcher 陣列，每個 matcher 含 `matcher`（正則字串）與 `hooks` 陣列（每項含 `type`、`command`、`timeout`），以確保寫入內容能被 Antigravity 正常讀取，並與可能存在的其他工具寫入內容並存。

#### Scenario: 寫入後保持格式相容

- **WHEN** 系統寫入或更新 marker 群組
- **THEN** 產生的 JSON 符合上述 schema，且不影響檔案中既有的非 SessionHub 群組

#### Scenario: 讀取既有相容格式

- **WHEN** hooks.json 已存在且含合法群組（如 `{ "群組名": { "enabled": true, "PreToolUse": [{ "matcher": "run_command", "hooks": [{ "type": "command", "command": "C:/x.bat", "timeout": 10 }] }] } }`）
- **THEN** 系統正確解析並在寫入 marker 群組時保留該群組不變

### Requirement: No real-time event pipeline in this change

系統 SHALL NOT 在本 change 中為 Antigravity 建立即時事件（bridge）管線；marker hook 僅作為「已安裝」狀態的識別標記，session 列表更新仍透過既有掃描機制（非事件推送）。

#### Scenario: 安裝後無需額外 watcher

- **WHEN** Antigravity 整合安裝完成
- **THEN** 系統不註冊對應的檔案 watcher 或即時事件監聽，session 資料更新沿用現有掃描／快取流程
