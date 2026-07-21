#!/usr/bin/env node
"use strict";

const path = require("path");
const {
    parseHookArgs,
    readHookPayload,
    writeBridgeEventRecord,
    writeHookErrorLog,
} = require(path.join(__dirname, "modules", "record-event.cjs"));

try {
    const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "copilot");
    if (!bridgePath) process.exit(0);

    const payload = readHookPayload();
    if (!payload) process.exit(0);

    let timestamp;
    if (payload.timestamp !== undefined && payload.timestamp !== null) {
        const ms = Number(payload.timestamp);
        if (Number.isFinite(ms)) {
            timestamp = new Date(ms).toISOString();
        }
    }

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "session.started",
        payload,
        timestamp,
    });
} catch (err) {
    writeHookErrorLog(err.message, "session.started");
}
