"use strict";

const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { handlePermissionRequest } = require("../../hooks/codex/on-permission-request.cjs");
const { handleNotification } = require("../../hooks/copilot/on-notification.cjs");

test("Codex only records a permission request when approval is needed", () => {
    const records = [];
    const notifications = [];
    handlePermissionRequest({
        payload: { session_id: "codex-1", cwd: "C:/project", tool_name: "Bash" },
        bridgePath: "unused",
        provider: "codex",
        recordEvent: (record) => records.push(record),
        notify: (notification) => notifications.push(notification),
    });
    assert.equal(records[0].eventType, "permission.requested");
    assert.equal(records[0].title, "Bash");
    assert.equal(notifications[0].kind, "intervention");
});

test("Copilot only records actual permission or elicitation prompts", () => {
    for (const notificationType of ["permission_prompt", "elicitation_dialog"]) {
        const records = [];
        const notifications = [];
        handleNotification({
            payload: { sessionId: "copilot-1", notification_type: notificationType },
            bridgePath: "unused",
            provider: "copilot",
            recordEvent: (record) => records.push(record),
            notify: (notification) => notifications.push(notification),
        });
        assert.equal(records[0].title, notificationType);
        assert.equal(notifications[0].kind, "intervention");
    }

    const records = [];
    const notifications = [];
    handleNotification({
        payload: { sessionId: "copilot-1", notification_type: "agent_idle" },
        bridgePath: "unused",
        provider: "copilot",
        recordEvent: (record) => records.push(record),
        notify: (notification) => notifications.push(notification),
    });
    assert.deepEqual(records, []);
    assert.deepEqual(notifications, []);
});

test("tool activity and completion hooks write recoverable session records", () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "sessionhub-overlay-hooks-"));
    try {
        for (const [provider, script, expectedEvent] of [
            ["codex", "on-pre-tool-use.cjs", "tool.pre"],
            ["codex", "on-session-end.cjs", "session.ended"],
            ["copilot", "on-pre-tool-use.cjs", "tool.pre"],
            ["copilot", "on-agent-stop.cjs", "session.idle"],
        ]) {
            const bridgePath = path.join(root, `${provider}.jsonl`);
            const scriptPath = path.join(__dirname, "..", "..", "hooks", provider, script);
            const result = spawnSync(process.execPath, [scriptPath, "--bridge-path", bridgePath, "--provider", provider], {
                input: JSON.stringify({ session_id: `${provider}-1`, sessionId: `${provider}-1`, cwd: "C:/project", tool_name: "Bash" }),
                encoding: "utf8",
            });
            assert.equal(result.status, 0, result.stderr);
            const records = fs.readFileSync(bridgePath, "utf8").trim().split("\n");
            const record = JSON.parse(records.at(-1));
            assert.equal(record.eventType, expectedEvent);
            assert.equal(record.sessionId, `${provider}-1`);
        }
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});
