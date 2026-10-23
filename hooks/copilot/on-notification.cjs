#!/usr/bin/env node
"use strict";

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
    if (notificationType !== "permission_prompt" && notificationType !== "elicitation_dialog") {
        return notificationType;
    }
    const sessionId = getHookStringValue(payload, ["sessionId", "session_id"]);
    recordEvent({
        bridgePath,
        provider,
        eventType: "notification",
        payload,
        title: notificationType,
    });
    notify({
        sessionId,
        title: "SessionHub — 需要您介入",
        body: notificationType === "permission_prompt"
            ? "Copilot 需要您確認工具使用授權"
            : "Copilot 正在等待您回覆問題",
        kind: "intervention",
    });
    return notificationType;
}

if (require.main === module) {
    try {
        const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "copilot");
        if (bridgePath) {
            const payload = readHookPayload();
            if (payload) handleNotification({ payload, bridgePath, provider });
        }
    } catch (err) {
        writeHookErrorLog(err.message, "notification");
    }
}

module.exports = { handleNotification };
