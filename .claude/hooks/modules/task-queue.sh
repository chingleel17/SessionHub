#!/usr/bin/env sh
# task-queue.sh - 任務佇列輔助模組（保留供後續擴充）

new_hook_task() {
    local type="$1"
    local payload="$2"
    printf '{"type":"%s","payload":%s}\n' "$type" "$payload"
}
