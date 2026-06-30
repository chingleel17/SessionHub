# Proposal: Tray Quota Widget

## 問題

SessionHub 現在可以抓取各 provider 的 quota 資訊（Claude 5h/7d、Copilot、OpenCode、Codex），但這些資訊只顯示在 Settings 或 Dashboard 頁面，使用者必須切換到 app 視窗才能確認用量。

Usage4Claude（macOS menu bar app）展示了更好的互動模式：用量直接反映在系統圖示上（icon 顏色/數字/小 bar），不需要開啟主視窗即可一眼掌握使用狀況。

## 提議解法

在 SessionHub 的系統匣圖示加入以下功能：

1. **System Tray 圖示動態顯示**：在系統匣圖示旁以文字或改變圖示顏色來反映最主要 provider（Claude）的當前 quota 使用率
2. **Tray 點擊彈出 Mini Panel**：點擊 tray 圖示時彈出一個精簡浮動視窗（mini popup），顯示所有 enabled provider 的 quota 狀態，類似 Usage4Claude 的 menu bar popover
3. **設定控制**：可設定顯示模式（圖示+數字 / 僅顏色指示 / 隱藏），可設定要顯示哪些 provider

## 參考

- **Usage4Claude**（macOS）：https://github.com/f-is-h/Usage4Claude — menu bar icon 隨用量變色，點擊展開詳細 popover
- **AirPodsDesktop**（Windows）：https://github.com/SpriteOvO/AirPodsDesktop — Windows 右下角彈出式 widget，使用 WinUI3 或 WPF，非常流暢的動畫

## 技術可行性

Tauri 2.x 已有 tray icon API，可以：
- 動態替換 tray icon image（用 Rust 動態繪製包含數字的 PNG）
- 設定 tooltip 顯示 quota 摘要
- 監聽 tray icon 點擊事件，開啟一個獨立的小視窗（Tauri window）

Windows 原生方式：
- 系統匣附近的彈出小視窗可用 Tauri 的 `WebviewWindowBuilder` 建立一個無框、置底的小視窗
- 也可考慮整合 `snoretoast` 的 balloon notification 顯示用量摘要（已有整合基礎）

## 預期效果

- 使用者在任何時間都能從系統匣看到 Claude 剩餘 quota，不需要開啟主視窗
- 點擊後展開精簡面板，顯示所有 provider 的 utilization bars + reset 倒數
- 顏色編碼：綠 &lt;50%、黃 50-80%、紅 &gt;80%
