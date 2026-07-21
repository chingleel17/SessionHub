"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const {
    handlePermissionRequest,
} = require("../../.claude/hooks/on-permission-request.cjs");
const {
    handleNotification,
} = require("../../.claude/hooks/on-notification.cjs");

function makeBridgePath() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "sessionhub-claude-hook-"));
    return { root, bridgePath: path.join(root, "claude.jsonl") };
}

test("PermissionRequest records only the tool name and sends intervention", () => {
    const { root, bridgePath } = makeBridgePath();
    const notifications = [];
    const payload = {
        session_id: "session-bash",
        cwd: "D:/private/project",
        tool_name: "Bash",
        tool_input: { command: "echo secret-value" },
    };

    handlePermissionRequest({
        payload,
        bridgePath,
        provider: "claude",
        notify: (notification) => notifications.push(notification),
    });

    const rawRecord = fs.readFileSync(bridgePath, "utf8").trim();
    const record = JSON.parse(rawRecord);
    assert.equal(record.eventType, "permission.requested");
    assert.equal(record.sessionId, "session-bash");
    assert.equal(record.title, "Bash");
    assert.equal(rawRecord.includes("secret-value"), false);
    assert.equal(notifications.length, 1);
    assert.equal(notifications[0].sessionId, "session-bash");
    assert.equal(notifications[0].kind, "intervention");
    fs.rmSync(root, { recursive: true, force: true });
});

test("PermissionRequest handles file and external-directory tools without filtering", () => {
    for (const toolName of ["Read", "Edit", "Write"]) {
        const notifications = [];
        handlePermissionRequest({
            payload: {
                session_id: `session-${toolName}`,
                tool_name: toolName,
                tool_input: { file_path: "D:/outside/private.txt" },
            },
            bridgePath: "unused",
            provider: "claude",
            recordEvent: () => {},
            notify: (notification) => notifications.push(notification),
        });
        assert.equal(notifications.length, 1);
        assert.equal(notifications[0].sessionId, `session-${toolName}`);
    }
});

test("Notification branches on notification_type only", () => {
    const cases = [
        ["permission_prompt", 1, "需要您授權"],
        ["idle_prompt", 1, "等待您回應"],
        ["auth_success", 0, null],
    ];

    for (const [notificationType, expectedCount, expectedTitle] of cases) {
        const notifications = [];
        const records = [];
        const result = handleNotification({
            payload: {
                session_id: "session-notification",
                notification_type: notificationType,
                matcher: "permission_prompt",
            },
            bridgePath: "unused",
            provider: "claude",
            recordEvent: (record) => records.push(record),
            notify: (notification) => notifications.push(notification),
        });

        assert.equal(result, notificationType);
        assert.equal(records[0].title, notificationType);
        assert.equal(notifications.length, expectedCount);
        if (expectedTitle) {
            assert.equal(notifications[0].title.includes(expectedTitle), true);
            assert.equal(notifications[0].sessionId, "session-notification");
        }
    }
});

test("PermissionRequest and Notification use the same session notification identity", () => {
    const notifications = [];
    const notify = (notification) => notifications.push(notification);
    const base = {
        bridgePath: "unused",
        provider: "claude",
        recordEvent: () => {},
        notify,
    };

    handlePermissionRequest({
        ...base,
        payload: { session_id: "same-session", tool_name: "Bash" },
    });
    handleNotification({
        ...base,
        payload: {
            session_id: "same-session",
            notification_type: "permission_prompt",
        },
    });

    assert.equal(notifications.length, 2);
    assert.equal(notifications[0].sessionId, "same-session");
    assert.equal(notifications[1].sessionId, "same-session");
});
