## Implementation Notes

> When implementing this change, we discovered that lib.rs already had a ProviderIntegrationStatus
> mechanism with version comparison logic. We reused and upgraded it instead of creating a new PluginStatus system.
> The task list below has been adjusted accordingly.

---

## 1. Rust Backend - Version Constant Upgrade

- [x] 1.1 Upgrade PROVIDER_INTEGRATION_VERSION from 1 to 2 in lib.rs
- [x] 1.2 Confirm ProviderIntegrationState enum contains: Installed, Outdated, Missing, ManualRequired, Error
- [x] 1.3 Update render_opencode_integration() to use correct opencode SDK event hook format (with import type Event from @opencode-ai/sdk)

## 2. Installed Plugin Fix

- [x] 2.1 Fix ~/.config/opencode/plugins/sessionhub-provider-event-bridge.ts: use event hook, set integrationVersion: 2

## 3. Frontend - Startup Version Detection Toast

- [x] 3.1 Add useRef(false) flag in src/App.tsx to ensure toast fires only once
- [x] 3.2 Add startup useEffect in src/App.tsx: after settings load, check providerIntegrations for outdated or missing; call showToast if found

## 4. Translations

- [x] 4.1 Add toast.providerOutdatedOnStartup key to src/locales/zh-TW.ts
- [x] 4.2 Add toast.providerOutdatedOnStartup key to src/locales/en-US.ts

## 5. Verification

- [x] 5.1 Run cargo build to confirm Rust compilation succeeds
- [x] 5.2 Run bun run build to confirm TypeScript type consistency
