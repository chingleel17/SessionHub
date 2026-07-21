#!/usr/bin/env sh
# db-ops.sh - 重試輔助模組

invoke_with_retry() {
    local cmd="$1"
    local max_attempts="${2:-3}"
    local delay_ms="${3:-50}"
    local attempt=0

    while [ "$attempt" -lt "$max_attempts" ]; do
        if eval "$cmd"; then
            return 0
        fi
        attempt=$((attempt + 1))
        if [ "$attempt" -ge "$max_attempts" ]; then
            return 1
        fi
        sleep "$(echo "scale=3; $delay_ms / 1000" | bc 2>/dev/null || echo 0.05)"
        delay_ms=$((delay_ms * 2))
        if [ "$delay_ms" -gt 500 ]; then
            delay_ms=500
        fi
    done
    return 1
}
