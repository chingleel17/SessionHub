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

try {
    const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "codex");
    if (!bridgePath) process.exit(0);

    const payload = readHookPayload();
    if (!payload) process.exit(0);

    const title = getHookStringValue(payload, ["tool_name"]);

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "tool.pre",
        payload,
        title,
    });

    const sessionId = getHookStringValue(payload, ["session_id", "sessionId"]);
    sendNotification({
        sessionId,
        title: "SessionHub — 需要您授權",
        body: title ? `Codex 即將執行工具：${title}` : "Codex 即將執行工具，需要您授權",
        kind: "intervention",
    });
} catch (err) {
    writeHookErrorLog(err.message, "tool.pre");
}
