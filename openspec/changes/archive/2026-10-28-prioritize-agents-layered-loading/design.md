# Design

## Context

動機見 proposal.md。既有 Skills 以完整來源設定與忽略路徑建立單一 query key，刷新／同步後更新已有請求合併與 dirty cache 機制；AGENTS.md 在 App.tsx 呼叫完整掃描。後端皆透過 spawn_blocking 掃描。

## Goals / Non-Goals

**Goals:** 根層先可操作、完整性不變、刷新保留資料且不混入其他上下文、同步結果可信。

**Non-Goals:** 不新增常駐磁碟索引、持久快取或預測性載入；不改 Commands/MCP 的掃描，不以猜測的同步狀態取代內容比對。

## Decisions

1. command 接受 optional `layer: root | nested | all`，未指定保持 all。AGENTS root 僅列根目錄，nested 排除根目錄但保留原深度與總預算；Skills root 只建立根目錄 context，不遍歷一般專案子樹，nested 沿用完整 context 探索但不重掃根 context 的 provider roots。全域維持完整固定入口掃描。
2. 使用一次邏輯 query 依序呼叫 root/nested，root 完成後以相同 key 發布中間結果，整體 fetchStatus 保持 fetching；nested 完成回傳合併資料。相比增加子 query keys，這可沿用原本精準刷新、dirty cache 與寫入競爭處理。共用工具不呼叫 IPC，IPC 仍集中於 App.tsx。每次發布前檢查 query AbortSignal，拒絕已取消的舊回應。
3. 前端結果以 optional `scanComplete` 區分中間與完整資料。root 發布時保留同 key 原有 nested 資料，但不把舊 root 復活；最終成功以新 root+nested 替換。MD 以實際 dir 排序，Skills 保留 context/name/provider；合併 metadata、診斷與計數。失敗保留中間結果並顯示錯誤，Skills 仍只接受手動重試。
4. global 查詢在對應專案 query 已結束後啟用；直接進入全域頁不受限制。既有全域 key 不變，已載入資料可立即顯示。背景補齊時保留已選文件，ContentViewer 僅依檔案本身讀取狀態顯示 loading。
5. 指示檔以一次 bytes 讀取同時計算 SHA-256 與 UTF-8 指標，保留既有 bytes hash 與非 UTF-8 指標空值語意。
6. OpenCode CLI discovery 延至 nested 階段，每次邏輯偵測只執行一次；nested 回傳 `cliSkills`，合併時按名稱大小寫不敏感補齊根層 OpenCode 技能及 discoveries。root 先回傳可信的檔案同步狀態，不提前宣稱 CLI available；不必重掃根層檔案。

## Risks / Trade-offs

- root 自身技能數很多仍有耗時 → 本次先移除整棵專案探索的首屏等待，保留正確同步判定，不虛構指紋結果。
- 中間資料被當作完整 → `scanComplete: false` 持續顯示補齊／失敗狀態，空根層不顯示確定空清單。
- 寫入與背景掃描競爭 → 沿用 refreshResource 的 afterWrite 等待加重掃，加入真實 QueryClient 回歸測試。
- 分層漏掃／重複 → Rust 測試比較兩層合併與 all 的等價性，涵蓋同名、深層、忽略、連結及預算。

## Migration Plan

無設定或資料庫遷移。既有 callers 省略 layer 繼續完整掃描；移除前端分層調度即可回復原行為。
