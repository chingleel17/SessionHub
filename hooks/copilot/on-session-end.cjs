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
    const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "copilot");
    if (!bridgePath) process.exit(0);

    const payload = readHookPayload();
    if (!payload) process.exit(0);

    const reason = getHookStringValue(payload, ["reason"]);
    const error = reason === "error" ? "copilot session ended with error" : "";

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "session.ended",
        payload,
        error,
    });

    const sessionId = getHookStringValue(payload, ["session_id", "sessionId"]);
    const cwd = getHookStringValue(payload, ["cwd"]);
    const projectName = cwd ? path.basename(cwd) : "";
    sendNotification({
        sessionId,
        title: "SessionHub — Session 已完成",
        body: projectName || "Copilot session 已結束",
        kind: "done",
    });
} catch (err) {
    writeHookErrorLog(err.message, "session.ended");
}
