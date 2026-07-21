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
    const { bridgePath, provider } = parseHookArgs(process.argv.slice(2), "claude");
    if (!bridgePath) process.exit(0);

    const payload = readHookPayload();
    if (!payload) process.exit(0);

    const prompt = getHookStringValue(payload, ["prompt"]);
    const title = prompt.slice(0, 80);

    writeBridgeEventRecord({
        bridgePath,
        provider,
        eventType: "prompt.submitted",
        payload,
        title,
    });
} catch (err) {
    writeHookErrorLog(err.message, "prompt.submitted");
}
