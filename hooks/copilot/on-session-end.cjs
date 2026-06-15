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

    const reason = getHookStringValue(payload, ["reason"]);
    const error = reason === "error" ? "copilot session ended with error" : "";

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "session.ended",
        payload,
        error,
    });
} catch (err) {
    writeHookErrorLog(err.message, "session.ended");
}
