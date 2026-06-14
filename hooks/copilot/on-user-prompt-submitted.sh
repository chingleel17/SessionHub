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
prompt="$(get_hook_string_value "prompt")"
title="$(printf '%s' "$prompt" | cut -c1-80)"
write_bridge_event_record "$BRIDGE_PATH" "$PROVIDER" "prompt.submitted" "$title"
