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

try {
    const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "copilot");
    if (!bridgePath) process.exit(0);

    const payload = readHookPayload();
    if (!payload) process.exit(0);

    const title = getHookStringValue(payload, ["toolName"]);
    const resultType =
        payload.toolResult && payload.toolResult.resultType
            ? String(payload.toolResult.resultType)
            : "";
    const error =
        resultType === "failure" || resultType === "denied"
            ? `tool ${title} ${resultType}`
            : "";

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "tool.post",
        payload,
        title,
        error,
    });
} catch (err) {
    writeHookErrorLog(err.message, "tool.post");
}
