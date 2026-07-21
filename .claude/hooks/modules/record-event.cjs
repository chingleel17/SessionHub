// record-event.cjs - Hook 事件記錄核心模組（Node.js 主軌，CommonJS）
// 不依賴外部工具（jq/bc/date），使用 Node 原生 JSON 與 Date
// 使用 .cjs 副檔名強制以 CommonJS 執行，不受落地目錄 package.json 的 type 影響

"use strict";

const fs = require("fs");
const path = require("path");
const os = require("os");

const BRIDGE_RECORD_VERSION = 4;

// 取得 SessionHub log 目錄（%APPDATA%/SessionHub/logs，非 Windows 退回 ~/AppData/Roaming）
function getSessionHubLogDir() {
    const appData =
        process.env.APPDATA || path.join(os.homedir(), "AppData", "Roaming");
    return path.join(appData, "SessionHub", "logs");
}

function ensureSessionHubLogDir() {
    const dir = getSessionHubLogDir();
    fs.mkdirSync(dir, { recursive: true });
    return dir;
}

// 將 hook 錯誤寫入 log，永不拋出（記錄失敗時靜默）
function writeHookErrorLog(message, eventType) {
    try {
        const logDir = ensureSessionHubLogDir();
        const logPath = path.join(logDir, "hook-errors.log");
        const entry = {
            timestamp: new Date().toISOString(),
            level: "error",
            eventType: eventType || "hook.error",
            message: String(message),
        };
        fs.appendFileSync(logPath, JSON.stringify(entry) + "\n", "utf8");
    } catch (_err) {
        // 記錄失敗時不阻斷流程
    }
}

// 從 stdin 同步讀取整個 payload 並解析；空白或解析失敗回傳 null
function readHookPayload() {
    let raw = "";
    try {
        raw = fs.readFileSync(0, "utf8");
    } catch (_err) {
        return null;
    }
    if (!raw || raw.trim().length === 0) {
        return null;
    }
    try {
        return JSON.parse(raw);
    } catch (err) {
        writeHookErrorLog("Failed to parse hook payload: " + err.message);
        return null;
    }
}

// 由 payload 依候選鍵名取出第一個非空字串值
function getHookStringValue(payload, propertyNames) {
    if (!payload || typeof payload !== "object") {
        return "";
    }
    for (const name of propertyNames) {
        const value = payload[name];
        if (value !== undefined && value !== null) {
            const str = String(value);
            if (str.trim().length > 0) {
                return str;
            }
        }
    }
    return "";
}

// 以遞增退避重試同步動作（對齊 psm1/sh：3 次、初始 50ms、上限 500ms）
function invokeWithRetry(action, maxAttempts, initialDelayMs) {
    const attempts = maxAttempts || 3;
    let delayMs = initialDelayMs || 50;
    for (let attempt = 1; attempt <= attempts; attempt++) {
        try {
            action();
            return;
        } catch (err) {
            if (attempt >= attempts) {
                throw err;
            }
            // 同步忙等退避（hook 為短命程序，避免引入非同步複雜度）
            const until = Date.now() + delayMs;
            while (Date.now() < until) {
                // busy-wait
            }
            delayMs = Math.min(delayMs * 2, 500);
        }
    }
}

// 將單筆 record append 至 bridge 檔（必要時建立父目錄）
function addBridgeRecord(bridgePath, record) {
    invokeWithRetry(() => {
        const parent = path.dirname(bridgePath);
        if (parent && parent !== ".") {
            fs.mkdirSync(parent, { recursive: true });
        }
        fs.appendFileSync(bridgePath, JSON.stringify(record) + "\n", "utf8");
    });
}

// 組裝並寫入標準 version 4 bridge record
function writeBridgeEventRecord(options) {
    const {
        bridgePath,
        provider,
        eventType,
        payload,
        title = "",
        error = "",
        timestamp,
    } = options;

    const sessionId = getHookStringValue(payload, ["session_id", "sessionId"]);
    const cwd = getHookStringValue(payload, ["cwd"]);
    const sourcePath = getHookStringValue(payload, [
        "transcript_path",
        "transcriptPath",
    ]);

    const record = {
        version: BRIDGE_RECORD_VERSION,
        provider,
        eventType,
        timestamp:
            timestamp && String(timestamp).trim().length > 0
                ? timestamp
                : new Date().toISOString(),
        sessionId,
        cwd,
        sourcePath,
        title,
        error,
    };

    addBridgeRecord(bridgePath, record);
}

// 解析 --bridge-path / --provider 旗標，其餘忽略
function parseHookArgs(argv, defaultProvider) {
    const result = { bridgePath: "", provider: defaultProvider || "" };
    for (let i = 0; i < argv.length; i++) {
        if (argv[i] === "--bridge-path") {
            result.bridgePath = argv[i + 1] || "";
            i++;
        } else if (argv[i] === "--provider") {
            result.provider = argv[i + 1] || result.provider;
            i++;
        }
    }
    return result;
}

module.exports = {
    parseHookArgs,
    readHookPayload,
    getHookStringValue,
    writeBridgeEventRecord,
    writeHookErrorLog,
};
