#!/usr/bin/env sh
# record-event.sh - Hook 事件記錄核心模組
# 依賴：jq（Git Bash 環境）

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/db-ops.sh"

# 確認 jq 可用，否則輸出錯誤並以 exit 0 結束（不阻斷 Claude）
_ensure_jq() {
    if ! command -v jq > /dev/null 2>&1; then
        printf '[SessionHub] hook 事件記錄需要 jq，但找不到 jq 執行檔。\n請安裝 jq：winget install jqlang.jq 或透過 Git for Windows 安裝程式勾選。\n' >&2
        exit 0
    fi
}

get_sessionhub_log_dir() {
    local appdata="${APPDATA:-$HOME/AppData/Roaming}"
    printf '%s/SessionHub/logs' "$appdata"
}

ensure_sessionhub_log_dir() {
    local dir
    dir="$(get_sessionhub_log_dir)"
    mkdir -p "$dir"
    printf '%s' "$dir"
}

write_hook_error_log() {
    local event_type="${1:-hook.error}"
    local message="$2"
    local log_dir
    log_dir="$(ensure_sessionhub_log_dir)"
    local log_path="$log_dir/hook-errors.log"
    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%S.000Z 2>/dev/null || printf 'unknown')"
    printf '{"timestamp":"%s","level":"error","eventType":"%s","message":"%s"}\n' \
        "$timestamp" "$event_type" "$(printf '%s' "$message" | sed 's/"/\\"/g')" \
        >> "$log_path"
}

# 從 stdin 讀取 hook payload JSON，存入全域變數 HOOK_PAYLOAD
read_hook_payload() {
    HOOK_PAYLOAD="$(cat)"
    if [ -z "$(printf '%s' "$HOOK_PAYLOAD" | tr -d '[:space:]')" ]; then
        HOOK_PAYLOAD=""
    fi
}

# 從 HOOK_PAYLOAD 取得指定欄位的字串值（第一個非空的欄位）
# 用法：get_hook_string_value "field1" "field2" ...
get_hook_string_value() {
    for field in "$@"; do
        local val
        val="$(printf '%s' "$HOOK_PAYLOAD" | jq -r --arg f "$field" '.[$f] // empty' 2>/dev/null)"
        if [ -n "$val" ] && [ "$val" != "null" ]; then
            printf '%s' "$val"
            return 0
        fi
    done
    printf ''
}

# 將事件記錄寫入 bridge JSONL 檔案
# 用法：write_bridge_event_record <bridge_path> <provider> <event_type> [title]
write_bridge_event_record() {
    local bridge_path="$1"
    local provider="$2"
    local event_type="$3"
    local title="${4:-}"

    local session_id cwd source_path timestamp
    session_id="$(get_hook_string_value "session_id" "sessionId")"
    cwd="$(get_hook_string_value "cwd")"
    source_path="$(get_hook_string_value "transcript_path" "transcriptPath")"
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%S.000Z 2>/dev/null || printf 'unknown')"

    local parent_dir
    parent_dir="$(dirname "$bridge_path")"
    if [ -n "$parent_dir" ] && [ "$parent_dir" != "." ]; then
        mkdir -p "$parent_dir"
    fi

    local record
    record="$(jq -cn \
        --argjson version 4 \
        --arg provider "$provider" \
        --arg eventType "$event_type" \
        --arg timestamp "$timestamp" \
        --arg sessionId "$session_id" \
        --arg cwd "$cwd" \
        --arg sourcePath "$source_path" \
        --arg title "$title" \
        '{version:$version,provider:$provider,eventType:$eventType,timestamp:$timestamp,sessionId:$sessionId,cwd:$cwd,sourcePath:$sourcePath,title:$title}'
    )"

    invoke_with_retry "printf '%s\n' '$record' >> '$bridge_path'"
}

_ensure_jq
