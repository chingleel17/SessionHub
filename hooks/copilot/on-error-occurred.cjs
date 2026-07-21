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

    const err = payload.error || {};
    const title = err.name ? String(err.name) : "";
    const error = err.message ? String(err.message) : "unknown error";

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "session.errored",
        payload,
        title,
        error,
    });
} catch (e) {
    writeHookErrorLog(e.message, "session.errored");
}
