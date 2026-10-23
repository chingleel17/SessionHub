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
    writeBridgeEventRecord({ bridgePath, provider, eventType: "session.idle", payload });
} catch (err) {
    writeHookErrorLog(err.message, "session.idle");
}
