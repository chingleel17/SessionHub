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

function handlePermissionRequest({
    payload,
    bridgePath,
    provider,
    recordEvent = writeBridgeEventRecord,
    notify = sendNotification,
}) {
    const sessionId = getHookStringValue(payload, ["session_id", "sessionId"]);
    const toolName = getHookStringValue(payload, ["tool_name", "toolName"]);

    recordEvent({
        bridgePath,
        provider,
        eventType: "permission.requested",
        payload,
        title: toolName,
    });
    notify({
        sessionId,
        title: "SessionHub — 需要您授權",
        body: toolName
            ? `Claude 需要您確認 ${toolName} 工具使用授權`
            : "Claude 需要您確認工具使用授權",
        kind: "intervention",
    });
}

if (require.main === module) {
    try {
        const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "claude");
        if (bridgePath) {
            const payload = readHookPayload();
            if (payload) {
                handlePermissionRequest({ payload, bridgePath, provider });
            }
        }
    } catch (err) {
        writeHookErrorLog(err.message, "permission.requested");
    }
}

module.exports = { handlePermissionRequest };
