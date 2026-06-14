#!/usr/bin/env sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BRIDGE_PATH=""
PROVIDER="copilot"

while [ $# -gt 0 ]; do
    case "$1" in
        --bridge-path) BRIDGE_PATH="$2"; shift 2 ;;
        --provider) PROVIDER="$2"; shift 2 ;;
        *) shift ;;
    esac
done

[ -n "$BRIDGE_PATH" ] || exit 0

. "$SCRIPT_DIR/modules/record-event.sh"
read_hook_payload
[ -n "$HOOK_PAYLOAD" ] || exit 0
ts_ms="$(get_hook_string_value "timestamp")"
timestamp="$(date -u -d "@$(expr "$ts_ms" / 1000)" +%Y-%m-%dT%H:%M:%S.000Z 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%S.000Z 2>/dev/null || printf 'unknown')"
write_bridge_event_record "$BRIDGE_PATH" "$PROVIDER" "session.started" "" "" "$timestamp"
