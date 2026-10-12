# Design

## Context

- 選單列圖示目前由 `app_setup::build_tray_icon` 以 `icons/32x32.png`（彩色）建立，之後 `tray_icon.rs` 依 quota 在底圖上疊彩色圓點／數字／進度條。未設定 template，因此 macOS 不會替它著色。
- App 圖示來自 `tauri.conf.json` 的 `bundle.icon`（`icon.icns` 等）。`.icns` 是單一外觀資源，系統無法產生灰色／tinted 變體。
- 現有顯示面：quota overlay（獨立 Tauri 視窗）、tray panel、主視窗狀態列。overlay 的資料由 `get_overlay_activity`（合併 hook 與 herdr 狀態）與 quota snapshot 查詢組成，目前只有 webview 讀得到。
- 本機（開發機）只安裝 Command Line Tools，沒有 Xcode：無 `actool`、無法建置 WidgetKit extension，也無法驗證 `.icon` 編譯產物。
- Windows 為主要目標平台，所有 macOS 行為須以 `cfg(target_os = "macos")` 隔離。

## Goals / Non-Goals

**Goals:**
- macOS 選單列圖示在淺色／深色／桌布亮度變化下對比正確。
- 打包後的 `.app` 在 macOS 26 的圖示風格切換下能換色。
- 抽出單一「活動快照」，讓 overlay、tray panel 與日後的原生 widget 共用。

**Non-Goals:**
- 不重寫現有 overlay／tray panel 的版面。
- 不做 iOS／iPadOS widget、不做 Windows widget 面板。
- 不在 `tauri dev` 的未簽章 binary 上支援圖示風格與 App Group（見風險）。

## Decisions

### D1. 選單列圖示：macOS 用 template image，單色繪製

macOS 上將底圖換成單色 alpha mask（黑色 + 透明），建立時設定 `icon_as_template(true)`，之後每次重繪 `set_icon_as_template(true)`。疊加內容也只用黑色與透明，用量等級以形狀表達：`icon_only` 用三段填充（空心／半實／實心）取代綠黃紅圓點；`percentage` 與 `bar` 直接以單色繪製。

- **替代方案**：保留彩色並依 `NSApp.effectiveAppearance` 切換深淺底圖。放棄原因：選單列顏色取決於桌布亮度而非系統外觀，會出現「深色模式 + 淺色選單列」的錯配；且 tinted 風格無法處理。
- **取捨**：失去綠／黃／紅色彩提示。以形狀＋既有 tooltip 補償；Windows 維持彩色不變。

### D2. App 圖示：以 Icon Composer 的 `.icon` 為來源，打包時編譯為 `Assets.car`

新增 `.icon` 資源，於 macOS 打包階段以 `actool` 編譯並放入 `.app/Contents/Resources`，Info.plist 設定 `CFBundleIconName`；保留 `icon.icns` 作舊系統退路。

- **替代方案**：只換一份單色 `.icns`。放棄原因：無法讓系統分別產生深色與 tinted 變體，也不會隨風格切換。
- **前提**：需要 Xcode（`actool`）。Tauri 2.12 的 bundler 是否原生支援 `.icon`、或需 `bundle.macOS` 後處理腳本，須在實作前以打包實測確認（見 tasks.md 任務 1.2）。若不支援，以打包後腳本注入 `Assets.car` 並重簽。

- **實測結果（2026-10-11，tauri-cli 2.10.1 / Tauri 2.12.1 / Xcode 27.0）**：`bun run tauri build --bundles app` 產出的 `.app` 只含 `Contents/Resources/icon.icns`，Info.plist 只有 `CFBundleIconFile`，沒有 `CFBundleIconName` 與 `Assets.car`；CLI 二進位中也找不到 `actool`／`Assets.car` 相關字串。結論：**bundler 不原生支援 `.icon`，需打包後腳本編譯並注入、修改 Info.plist 後重簽**。
- **環境**：`/Applications/Xcode.app` 存在但 `xcode-select` 指向 Command Line Tools，需 `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`（或 `sudo xcode-select -s`）才能使用 `actool`。

### D3. 活動快照：後端單一函式 + 共用檔案

把 `get_overlay_activity` 的合併邏輯與 quota 快照組成 `ActivitySnapshot`（quota、等待清單、處理中數量、`generatedAt`），由一個函式產生。overlay 與 tray panel 呼叫同一函式；狀態變動時（沿用既有的 `intervention-list-changed`、`active-session-count-changed`、`quota-snapshots-updated` 事件）節流後寫入 `activity-snapshot.json`（原子寫入：暫存檔後 rename）。

- 檔案位置：App Group 容器（供原生 widget 讀取）；無 App Group 時退回 `app_data_dir`，行為不變。
- **替代方案**：widget 直接開 SQLite 或呼叫 CLI。放棄原因：widget extension 受沙盒限制、啟動成本高，且會把狀態合併邏輯在 Swift 再實作一份。

- **實測結果（2026-10-11）**：本機 `security find-identity -v -p codesigning` 為 0 個有效身分，打包產物僅 ad-hoc／linker-signed（`TeamIdentifier=not set`、`codesign --verify --deep --strict` 失敗：`code has no resources but signature indicates they must be present`）。App Group entitlement 需要 Apple Developer Team ID 簽章，此環境無法驗證寫入 App Group 容器。快照路徑解析已實作為「容器存在才使用，否則退回 `app_data_dir`」，因此無簽章環境行為不變。

### D4. 小工具路線：先交付 Tauri 路線，原生 WidgetKit 以可行性驗證決定

- 先完成 D3，使現有 overlay／tray panel 共用同一資料來源（路線 A，立即可用）。
- 原生 WidgetKit（路線 B）列為驗證任務：確認 (1) Tauri 打包流程可嵌入 `.appex` 並重簽，(2) App Group entitlement 在目前簽章方式下可用，(3) 開發機可取得 Xcode。三項皆成立才實作 widget；任一不成立則以路線 A 結案，並把結果記錄在本 change。
- **替代方案**：直接做 WidgetKit。放棄原因：本機沒有 Xcode，且 Tauri 打包與 extension 簽章整合風險高，先驗證可避免投入後才發現無法交付。

## Risks / Trade-offs

- [template 模式失去顏色提示] → 以形狀三段與 tooltip 補償，Windows 不受影響。
- [本機無 Xcode，無法驗證 `.icon` 與 widget] → 驗證任務放在最前面；沒有 Xcode 時 D2／路線 B 以文件與腳本備妥，並在 CI 或有 Xcode 的機器驗證。
- [`tauri dev` 看不到圖示風格與 App Group] → 驗證一律用打包後的 `.app`，並在任務中明寫。
- [快照檔被多個寫入者競爭或殘留舊資料] → 原子寫入、附 `generatedAt`，讀取端以有效期限判斷過期（見 `activity-snapshot-feed` spec）。
- [App Group 需要簽章身分] → 無簽章時退回 `app_data_dir`，原生 widget 路線標為不可用而非失敗。
