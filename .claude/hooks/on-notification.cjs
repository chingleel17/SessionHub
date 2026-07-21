#!/usr/bin/env node
"use strict";

// Claude Notification hook：處理 permission_prompt（需授權）與 idle_prompt（等待回應）
// https://docs.anthropic.com/zh-TW/docs/claude-code/hooks#notification

const path = require("path");
const {
    parseHookArgs,
    readHookPayload,
    getHookStringValue,
    writeBridgeEventRecord,
    writeHookErrorLog,
} = require(path.join(__dirname, "modules", "record-event.cjs"));
const { sendNotification } = require(path.join(__dirname, "modules", "notify.cjs"));

function handleNotification({
    payload,
    bridgePath,
    provider,
    recordEvent = writeBridgeEventRecord,
    notify = sendNotification,
}) {
    const notificationType = getHookStringValue(payload, ["notification_type"]);
    const sessionId = getHookStringValue(payload, ["session_id", "sessionId"]);

    recordEvent({
        bridgePath,
        provider,
        eventType: "notification",
        payload,
        title: notificationType,
    });

    if (notificationType === "permission_prompt") {
        notify({
            sessionId,
            title: "SessionHub — 需要您授權",
            body: "Claude 需要您確認工具使用授權",
            kind: "intervention",
        });
    } else if (notificationType === "idle_prompt") {
        notify({
            sessionId,
            title: "SessionHub — 等待您回應",
            body: "Claude 正在等待您的決策或回覆",
            kind: "intervention",
        });
    }

    return notificationType;
}

if (require.main === module) {
    try {
        const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "claude");
        if (bridgePath) {
            const payload = readHookPayload();
            if (payload) {
                handleNotification({ payload, bridgePath, provider });
            }
        }
    } catch (err) {
        writeHookErrorLog(err.message, "notification");
    }
}

module.exports = { handleNotification };
