// notify.cjs - Hook 系統通知模組（Windows Only，使用 snoretoast）
// 依 kind 讀取 settings.json 開關決定是否發送，失敗時靜默寫 hook-errors.log
"use strict";

const fs = require("fs");
const path = require("path");
const os = require("os");
const childProcess = require("child_process");

const SNORETOAST_EXE_NAME = "snoretoast.exe";
const SESSION_HUB_APP_ID = "SessionHub";
const SETTINGS_FILE = "settings.json";

function getAppDataDir() {
    return process.env.APPDATA || path.join(os.homedir(), "AppData", "Roaming");
}

function getSessionHubLogDir() {
    return path.join(getAppDataDir(), "SessionHub", "logs");
}

function writeErrorLog(message) {
    try {
        const logDir = getSessionHubLogDir();
        fs.mkdirSync(logDir, { recursive: true });
        const logPath = path.join(logDir, "hook-errors.log");
        const entry = {
            timestamp: new Date().toISOString(),
            level: "error",
            eventType: "notify.error",
            message: String(message),
        };
        fs.appendFileSync(logPath, JSON.stringify(entry) + "\n", "utf8");
    } catch (_err) {
        // 寫 log 失敗時靜默
    }
}

function readSettings() {
    try {
        const settingsPath = path.join(getAppDataDir(), "SessionHub", SETTINGS_FILE);
        const raw = fs.readFileSync(settingsPath, "utf8");
        return JSON.parse(raw);
    } catch (_err) {
        return null;
    }
}

function isNotificationEnabled(kind) {
    const settings = readSettings();
    if (!settings) {
        // 讀取失敗：安全預設（介入開、結束關）
        return kind === "intervention";
    }
    if (kind === "done") {
        const val = settings.enableSessionEndNotification;
        return val === undefined ? false : Boolean(val);
    }
    // kind === "intervention"
    const val = settings.enableInterventionNotification;
    return val === undefined ? true : Boolean(val);
}

function findSnoretoast() {
    // 落地目錄：notify.cjs 在 <hook_root>/modules/，snoretoast.exe 在 <hook_root>/_bin/
    // 開發目錄：notify.cjs 在 hooks/<provider>/modules/，snoretoast.exe 在 hooks/_bin/
    const candidates = [
        path.join(__dirname, "..", "_bin", SNORETOAST_EXE_NAME),
        path.join(__dirname, "..", "..", "_bin", SNORETOAST_EXE_NAME),
    ];
    for (const p of candidates) {
        if (fs.existsSync(p)) return p;
    }
    return null;
}

/**
 * 發送 Windows 系統 Toast 通知
 * @param {object} options
 * @param {string} options.sessionId - 用於通知去重的 session id
 * @param {string} options.title - 通知標題
 * @param {string} options.body - 通知內容
 * @param {"done"|"intervention"} options.kind - 通知語意類型
 */
function sendNotification({ sessionId, title, body, kind }) {
    try {
        if (process.platform !== "win32") return;

        if (!isNotificationEnabled(kind)) return;

        const snoretoast = findSnoretoast();
        if (!snoretoast) {
            writeErrorLog("snoretoast.exe not found, cannot send notification");
            return;
        }

        const notifId = `sessionhub-${sessionId || "unknown"}`;
        const args = [
            "-t", title,
            "-m", body,
            "-appID", SESSION_HUB_APP_ID,
            "-id", notifId,
            "-silent",
        ];

        childProcess.spawnSync(snoretoast, args, {
            timeout: 5000,
            windowsHide: true,
        });
    } catch (err) {
        writeErrorLog("sendNotification failed: " + err.message);
    }
}

module.exports = { sendNotification };
