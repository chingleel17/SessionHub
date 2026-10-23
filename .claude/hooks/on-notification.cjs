#!/usr/bin/env node
"use strict";

// Claude Notification hook：僅處理真正需要授權的 permission_prompt
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
    if (notificationType !== "permission_prompt") return notificationType;
    const sessionId = getHookStringValue(payload, ["session_id", "sessionId"]);

    recordEvent({
        bridgePath,
        provider,
        eventType: "notification",
        payload,
        title: notificationType,
    });

    notify({
        sessionId,
        title: "SessionHub — 需要您授權",
        body: "Claude 需要您確認工具使用授權",
        kind: "intervention",
    });

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
