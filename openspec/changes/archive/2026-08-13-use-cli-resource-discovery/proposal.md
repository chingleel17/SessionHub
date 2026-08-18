## Why

Agents 頁目前以目錄與設定檔掃描結果推測 Skills、Commands 與 MCP 是否「已載入／未安裝」，但檔案存在或同步一致不代表 CLI 實際解析並啟用了該資源，造成狀態標示失真。Skills 閱讀器的玻璃背景也讓底層清單文字穿透，降低長文閱讀對比。

## What Changes

- 僅對已實測具有穩定、可機器解析介面的 provider/resource 組合使用原生 CLI：OpenCode Skills、OpenCode Commands、Copilot MCP 與 Codex MCP；在專案工作目錄與全域中立目錄執行，以納入 provider 實際的設定合併與 scope 規則。
- Skills、Commands 與 MCP 的 provider 集合 SHALL 由 `AppSettings.enabledProviders` 決定，不再固定建立四平台欄位；停用平台不掃描、不執行 CLI，也不出現在狀態清單或同步目標。
- 專案與全域 Skills／Commands 改為 provider-aware root 掃描：每個啟用平台只掃描該工具正式支援或既有相容行為涵蓋的目錄，合併同名項目時保留 provider 與來源 root，不再以單一 `.agents` 正本加同步目標代表各工具實際可見資源。
- 將檔案同步狀態與 CLI 執行狀態拆成不同欄位與文案；不再由 `in-sync`、`target-missing` 或偏好設定推導「已載入／未安裝」。
- 對未通過能力驗證的 provider/resource 組合完全跳過 CLI 檢查，不實作脆弱的文字 parser、不顯示 unknown/error badge，也不將檔案掃描結果冒充執行狀態；既有預覽、同步與設定管理能力維持不變。
- 支援 CLI 檢查的 Skills、Commands 與 MCP 清單顯示資料來源、scope 與查詢結果；首次切入對應頁籤、手動重新整理，以及同步或 MCP 變更成功後才重新檢查，不背景輪詢。
- MCP 清單保留現有設定編輯能力；僅 Copilot 與 Codex 顯示 CLI 可驗證的 effective/configured 狀態，OpenCode 與 Claude MCP 維持設定檔管理，不宣稱 runtime 連線狀態。
- Skills／Commands 閱讀器改用不透明的主題 surface，確保 light、dark 主題下內容不受底層文字干擾，並維持 modal 內部捲動。

## Capabilities

### New Capabilities

<!-- 無；此變更修正既有資源管理與檢視能力。 -->

### Modified Capabilities

- `agents-skills-sync`: 依啟用 provider 的實際相容 roots 掃描 Skills；OpenCode 另以 CLI effective 結果驗證。
- `agents-commands-sync`: 依啟用 provider 的實際 command roots 掃描 Commands；OpenCode 另以 resolved config 驗證。
- `mcp-config-management`: Copilot 與 Codex MCP 清單合併可機器解析的 CLI effective 狀態，其他 provider 維持設定檔管理。
- `agents-config-view-ux`: 清單狀態語意與圖例改為反映 CLI 查詢結果，閱讀器使用不透明內容 surface。

## Impact

- 後端：`src-tauri/src/agents_config.rs`、`src-tauri/src/mcp_config.rs`、對應 command 與測試；新增 provider root registry 與受逾時、工作目錄及輸出解析保護的 CLI adapter。
- 前端：`src/App.tsx`、`src/types/index.ts`、`src/components/AgentsConfigView.tsx`、`src/components/McpConfigView.tsx`、翻譯與 `src/App.css`。
- 外部系統：只呼叫本機已安裝的 opencode、copilot、codex CLI 白名單唯讀子命令；不新增套件依賴，不修改使用者資源設定。
- 相容性：既有同步與 MCP 寫入流程保留；支援項目的 CLI 不存在或輸出無法解析時只隱藏該次 CLI 狀態並顯示非阻擋提示，不讓整頁載入失敗。
