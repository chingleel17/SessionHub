#!/usr/bin/env sh
# on-stop.sh - Claude Stop hook

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

BRIDGE_PATH=""
PROVIDER="claude"

while [ $# -gt 0 ]; do
    case "$1" in
        --bridge-path) BRIDGE_PATH="$2"; shift 2 ;;
        --provider)    PROVIDER="$2";    shift 2 ;;
        *) shift ;;
    esac
done

if [ -z "$BRIDGE_PATH" ]; then
    printf '[SessionHub] --bridge-path is required\n' >&2
    exit 0
fi

. "$SCRIPT_DIR/modules/record-event.sh"

read_hook_payload
if [ -z "$HOOK_PAYLOAD" ]; then exit 0; fi

# stop_reason 放在 title 欄位，供後端判斷 idle vs waiting
stop_reason="$(get_hook_string_value "stop_reason" "stopReason")"
write_bridge_event_record "$BRIDGE_PATH" "$PROVIDER" "session.stop" "$stop_reason"
