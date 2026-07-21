## Context

`App.tsx` 目前結構（由上而下）：
1. `EmbeddedQuotaOverlayApp`（67-150 行）：獨立 webview 用元件，訂閱 `quota-snapshots-updated` / `quota-overlay-settings-changed` / `quota-overlay-locked-changed` 三個事件，渲染 `<QuotaOverlay>`
2. `EmbeddedTrayPanelApp`（152-210 行）：獨立 webview 用元件，訂閱 `quota-snapshots-updated`，處理 Escape 鍵隱藏視窗，渲染 `<TrayQuotaPanel>`
3. 一批 module-level helper 函式（212-458 行）
4. 主 `App()` 元件（461-2754 行），內含設定表單相關 state/mutations
5. `RoutedApp()`（2756-2766 行）：依 `EMBEDDED_VIEW` 分派上述三者之一

`EMBEDDED_VIEW` 是在 module 頂層（第 61 行）用 `URLSearchParams` 讀取的常數，三個元件都依賴它決定是否套用 `embedded-quota-view` CSS class。這個 module-level 副作用（`document.documentElement.classList.add`）需要保留在同一個「先執行」的位置。

設定表單邏輯目前分散於 `App()` 內：`settingsForm` state（522）、映射 `useEffect`（639）、`settingsMutation`（819）、`persistSettingsSilently`（1054）、`buildSettingsPayload`（998）、`detectTerminalMutation`（879）、`detectVscodeMutation`（891）、`providerIntegrationMutation`（903）。這些都圍繞同一份 `settingsForm` state 運作，具備抽成單一 hook 的內聚性。

**與既有 spec 的關係（重要，範圍較 change 2 更需注意）**：`openspec/specs/mcp-config-management/spec.md:206` 規定「前端 IPC 呼叫 SHALL 集中於 `App.tsx`」。本 change 涉及兩類搬移，性質不同：
- `useAppSettingsForm()` hook：與 change 2 的 `useSessionRealtimeEvents()` 同性質，僅被 `App()` 呼叫、非展示元件，依先前使用者決策視為符合該條文精神，不算 delta。
- `EmbeddedQuotaOverlayApp` / `EmbeddedTrayPanelApp`：這兩者**本身就是獨立 React 元件**（會被 `RoutedApp` 渲染），且原本就直接呼叫 `invoke()` / `listen()`（例如 `invoke<AppSettings>("get_settings")`、`invoke("save_settings", ...)`）。它們搬到獨立檔案後，IPC 呼叫點確實字面上脫離 `App.tsx` 這個檔案。但這兩個元件從一開始就**不在** `App()` 元件樹內、也不是 `mcp-config-management` spec 條文語境所指的「`App.tsx` 內主應用渲染出的子元件」——它們是完全獨立的 webview 進入點（`EMBEDDED_VIEW` 分流），與主應用的 `App()` 元件是平行關係而非父子關係，早在搬移之前就已經是「另一個獨立的小型 app」，只是物理上恰好也寫在同一個檔案裡。因此本 change 判定：搬移這兩個元件不屬於「IPC 下放到子元件」的既有規範所要防範的情況，不構成 delta。此判斷風險略高於 change 2，若執行時對此判讀有疑慮，應暫停並重新確認，不可自行擴大解釋範圍到其他情境。

## Goals / Non-Goals

**Goals:**
- 把兩個 embedded 元件搬到獨立檔案，`App.tsx` 不再需要知道它們的實作細節
- 把設定表單邏輯抽成 `useAppSettingsForm()` hook，回傳 `{ settingsForm, settingsMutation, buildSettingsPayload, ... }` 供 `App()` 使用
- `RoutedApp` 的分派邏輯與 `EMBEDDED_VIEW` 判斷保持功能不變
- 複用 change `cleanup-deps-and-settings-defaults` 產出的 `DEFAULT_APP_SETTINGS` / `mergeAppSettings()`，不重新手寫預設值

**Non-Goals:**
- 不改變 embedded webview 的建立方式（`create_quota_overlay`、tray panel 視窗管理留在後端 `lib.rs`，本次不動）
- 不處理 `extract-app-tsx-event-hooks` change 範圍內的事件監聽重構（兩者可能有少量重疊——`quota-snapshots-updated` 在主 `App()` 與 embedded 元件都有各自訂閱——但本次不合併主應用與 embedded 元件的訂閱邏輯，因為它們是不同 React tree/webview，無法共用同一個 hook 實例）
- 不改變 `settingsForm` 欄位結構或驗證規則

## Decisions

### D1：embedded 元件搬到 `src/app/` 新目錄，`EMBEDDED_VIEW` 判斷留在最上層 entry
新增 `src/app/` 目錄放置：
- `src/app/EmbeddedQuotaOverlayApp.tsx`
- `src/app/EmbeddedTrayPanelApp.tsx`
- `src/app/RoutedApp.tsx`（內含 `EMBEDDED_VIEW` 常數讀取、`document.documentElement.classList.add` 副作用、分派邏輯）

`src/App.tsx` 只保留主 `App()` 元件並改為 named export（或維持 default export，視現有 import 慣例調整），不再 export `RoutedApp`。`src/main.tsx` 改為 `import RoutedApp from "./app/RoutedApp"`。

替代方案（放棄）：把三個元件都塞進同一個檔案 `src/app/EmbeddedApps.tsx`。放棄原因：`RoutedApp` 是路由層而非「embedded app」本身，混在一起會讓檔案職責不清；分開三個小檔案對這個專案既有的 `components/` 慣例（一元件一檔）更一致。

### D2：`useAppSettingsForm()` 回傳完整表單控制介面，`App()` 解構使用
```ts
// src/hooks/useAppSettingsForm.ts
function useAppSettingsForm(params: {
  settingsQuery: UseQueryResult<AppSettings>;
  pinnedProjects: string[];
  showToast: (msg: string) => void;
  onSettingsSaved: () => void; // 觸發 restart_session_watcher 等外部副作用
}): {
  settingsForm: AppSettings;
  setSettingsForm: Dispatch<SetStateAction<AppSettings>>;
  buildSettingsPayload: (overrides?: Partial<AppSettings>) => AppSettings;
  persistSettingsSilently: (next: AppSettings) => Promise<void>;
  settingsMutation: UseMutationResult<...>;
  detectTerminalMutation: UseMutationResult<...>;
  detectVscodeMutation: UseMutationResult<...>;
  providerIntegrationMutation: UseMutationResult<...>;
}
```
`onSettingsSaved` 這個 callback 讓 hook 不需要知道 `restart_session_watcher` 等與「設定表單」邏輯上無關、但目前耦合在 `settingsMutation.onSuccess` 裡的副作用該由誰觸發——保留由 `App()` 傳入 callback 的方式，避免 hook 過度膨脹去處理 watcher 重啟這種不屬於「表單」職責的邏輯。

替代方案（放棄）：把 `restart_session_watcher` 呼叫也塞進 hook 內部。放棄原因：那是 session watcher 生命週期管理的職責，不屬於「設定表單」，混入會讓 hook 職責界線模糊，且與 `extract-app-tsx-event-hooks` change 的事件/watcher 相關重構產生範圍重疊。

## Risks / Trade-offs

- **[風險] `EMBEDDED_VIEW` 常數與 `document.documentElement.classList.add` 副作用若在模組載入順序上被打亂，可能導致 embedded webview 樣式閃爍或遺漏 class** → 緩解：把這段副作用整段搬到 `RoutedApp.tsx` 頂層（module scope），保持與原本相同的「模組載入時立即執行」時機，tasks.md 要求驗證兩個 embedded webview 開啟時樣式正常
- **[風險] `useAppSettingsForm` 抽出時遺漏某個 mutation 的 `onSuccess`/`onError` 副作用（例如 toast 文案、`queryClient.invalidateQueries` 的 queryKey）** → 緩解：tasks.md 要求逐一比對每個 mutation 搬移前後的 callback 內容
- **[風險] `App()` 與 hook 之間傳遞的 callback（`onSettingsSaved`）若遺漏某個依賴，可能造成 watcher 未正確重啟** → 緩解：手動測試變更 provider root 設定後確認 watcher 重啟生效

## Migration Plan

無資料遷移。純前端程式碼重構與檔案搬移，一般 PR 流程套用。可透過還原單一 commit rollback。
