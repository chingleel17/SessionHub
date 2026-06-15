## Why

Session Card 的複製功能目前無法根據不同平台產生對應的 CLI 工具指令。不論 session 來自哪個平台（Copilot、OpenCode、Codex、Claude），複製出來的指令都是固定的 `copilot --resume=<id>`，導致用戶複製後無法在正確的平台中執行該指令。

## What Changes

- 修改 `handleCopyCommand` 函式，根據 session 的 provider 類型生成對應的 CLI 工具指令
- 支援以下平台的指令生成：
  - **Copilot**: `copilot --resume=<session-id>`
  - **OpenCode**: `opencode session <session-id>`
  - **Codex**: `codex session <session-id>` 或其他適當的 Codex CLI 指令
  - **Claude**: `claude code --session=<session-id>` 或其他適當的 Claude CLI 指令

## Capabilities

### New Capabilities
- `platform-specific-copy-command`: 根據 session 的 provider 類型動態生成對應 CLI 工具的開啟指令

### Modified Capabilities
- `session-copy-functionality`: 改進現有的 session 複製功能，使其針對不同平台返回正確的指令

## Impact

- **Frontend**: `src/App.tsx` 中的 `handleCopyCommand` 函式需要修改
- **Affected Components**: SessionCard 相關功能不變，只改變複製指令的邏輯
- **Localization**: 可能需要新增 toast 訊息用於不支援複製的平台
