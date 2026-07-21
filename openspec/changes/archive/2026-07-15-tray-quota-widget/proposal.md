# Proposal: Tray Quota Widget

## 問題

SessionHub 現在可以抓取各 provider 的 quota 資訊（Claude 5h/7d、Copilot、OpenCode、Codex），但這些資訊只顯示在 Settings 或 Dashboard 頁面，使用者必須切換到 app 視窗才能確認用量。

理想的使用體驗是像 FPS Monitor / MSI Afterburner 的 OSD 那樣：一個**常駐螢幕角落、永遠置頂、不被其他視窗遮蓋**的小型 overlay widget，寫程式時瞄一眼就能掌握各 provider 的剩餘 quota，完全不需要切換視窗。

## 提議解法

三層顯示，由淺到深：

1. **System Tray 圖示動態顯示**：系統匣圖示以顏色/數字/小 bar 反映主要 provider（Claude）的當前 quota 使用率，hover 顯示 tooltip 摘要
2. **Overlay Widget（本變更主體）**：常駐桌面的無框透明小視窗，永遠置頂、不搶焦點，顯示各 enabled provider 的 utilization bar；支援「鎖定模式」（滑鼠穿透、純顯示）與「編輯模式」（可拖曳調整位置），位置跨重啟記憶
3. **Tray 點擊彈出 Mini Panel**：點擊 tray 圖示彈出精簡浮動面板，顯示完整 quota 詳情（含 reset 倒數、錯誤狀態、立即刷新），失焦自動隱藏——作為 overlay 的補充互動層
4. **設定控制**：tray 圖示顯示模式、overlay 開關/透明度/顯示哪些 provider、panel 開關

## 參考

- **MSI Afterburner / RTSS**：目標視覺體驗（常駐 OSD）。註：其 in-game overlay 是靠 DirectX hook 注入，一般桌面 app 做不到，本變更採 always-on-top 視窗方案（見「已知限制」）
- **Usage4Claude**（macOS）：https://github.com/f-is-h/Usage4Claude — menu bar icon 隨用量變色，點擊展開 popover（對應第 1、3 層）
- **Seelen UI**：https://github.com/eythaann/Seelen-UI — Rust + Tauri 的 Windows 桌面 shell，最活躍的 Tauri overlay 開源專案，視窗疊加手法可參考
- **Manasight blog**：https://blog.manasight.gg/why-i-chose-tauri-v2-for-a-desktop-overlay/ — Tauri v2 桌面 overlay 第一手實測（click-through、焦點問題）

## 技術可行性

Tauri 2.x 已具備所有必要 API：

- Overlay 視窗：`transparent(true)` + `decorations(false)` + `always_on_top(true)` + `skip_taskbar(true)` + `shadow(false)`
- 滑鼠穿透：`set_ignore_cursor_events(bool)`（整窗開關，搭配鎖定/編輯模式）
- 不搶焦點：`WebviewWindowBuilder::focused(false)`（**必須在 Rust 端建立**，`tauri.conf.json` 的 `focus: false` 在 Windows 上有已知 bug 不生效，tauri#11566 / #7519）
- 位置記憶：官方 `tauri-plugin-window-state`（內建多螢幕邊界驗證）
- Tray 圖示：動態替換 icon image（Rust 動態繪製 PNG）、tooltip、點擊事件

## 已知限制（重要）

- **蓋不過 exclusive fullscreen（獨佔全螢幕）應用**：Windows z-order band 機制限制，任何一般視窗（含 PowerToys Always on Top）皆無法蓋過獨佔全螢幕遊戲；borderless windowed 模式則正常。Afterburner 的 in-game OSD 是 DX hook 方案，超出本工具範疇且有反作弊風險，不採用
- **Windows transparent 視窗已知 bug**：初始化白底需 resize 一次才透明（tauri#4881）、拖曳可能出現殘影（#14764），需在實作中加入 workaround
- click-through 只能整窗開關，無法區域級穿透（tauri#2090）

## 預期效果

- Overlay widget 常駐螢幕角落，任何時候（除獨佔全螢幕外）都能看到各 provider 的 quota bar 與百分比
- 鎖定時滑鼠可穿透 widget 操作底下的視窗；編輯時可拖曳到任意位置，位置跨重啟保留
- 系統匣圖示同步反映用量；點擊展開詳細 panel
- 顏色編碼：綠 <50%、黃 50-80%、紅 >80%
