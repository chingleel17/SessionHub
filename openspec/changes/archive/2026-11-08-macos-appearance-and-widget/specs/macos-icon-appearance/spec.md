# Spec Delta

## Purpose

讓 SessionHub 在 macOS 上的 App 圖示與選單列圖示跟隨系統的圖示風格與外觀設定，避免使用者切換灰色、深色或 tinted 風格後仍看到不一致的圖示。

## ADDED Requirements

### Requirement: App 圖示跟隨 macOS 圖示風格

系統在 macOS 打包產物中 SHALL 提供支援多外觀的 App 圖示，使 Dock 與 Launchpad 的圖示能隨使用者選擇的圖示風格（預設、深色、clear／tinted）改變。

#### Scenario: 切換為灰色或 tinted 風格

- **WHEN** 使用者在 macOS 系統設定將圖示風格切換為深色、clear 或 tinted，且 SessionHub 以打包後的 `.app` 執行
- **THEN** Dock 中的 SessionHub 圖示依該風格重新著色，與其他支援的 app 一致
- **AND** 圖示主體不因風格切換而被裁切或失去辨識度

#### Scenario: 不支援多外觀的系統版本

- **WHEN** 執行於不支援多外觀圖示的 macOS 版本
- **THEN** 系統顯示預設（彩色）App 圖示，行為與目前相同

### Requirement: 選單列圖示以 template 方式呈現

系統在 macOS 上 SHALL 以 template image 呈現選單列（tray）圖示，使圖示顏色由系統依淺色／深色外觀與桌布亮度決定。

#### Scenario: 淺色與深色選單列

- **WHEN** 選單列因外觀或桌布亮度而呈現淺色或深色
- **THEN** SessionHub 選單列圖示自動以對應的對比色顯示，不出現固定彩色底圖造成的低對比

#### Scenario: 非 macOS 平台

- **WHEN** 執行於 Windows 或其他非 macOS 平台
- **THEN** 系統匣圖示維持目前的彩色底圖與疊加繪製行為，不受此需求影響
