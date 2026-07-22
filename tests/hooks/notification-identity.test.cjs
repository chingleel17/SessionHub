"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const EXPECTED_APP_ID = "com.ching.sessionhub";
const NOTIFY_MODULES = [
    ".claude/hooks/modules/notify.cjs",
    "hooks/codex/modules/notify.cjs",
    "hooks/copilot/modules/notify.cjs",
];

test("all provider notification assets use the SessionHub Windows identity", () => {
    for (const relativePath of NOTIFY_MODULES) {
        const content = fs.readFileSync(path.join(__dirname, "..", "..", relativePath), "utf8");

        assert.match(
            content,
            new RegExp(`const SESSION_HUB_APP_ID = "${EXPECTED_APP_ID}";`),
            relativePath,
        );
        assert.match(content, /"-appID", SESSION_HUB_APP_ID/, relativePath);
        assert.match(content, /const notifId = `sessionhub-\$\{sessionId \|\| "unknown"\}`;/, relativePath);
        assert.match(content, /const SESSION_HUB_TOAST_IMAGE_NAME = "sessionhub-logo\.png";/, relativePath);
        assert.match(content, /if \(toastImage\) args\.push\("-p", toastImage\);/, relativePath);
        assert.match(content, /"-silent",/, relativePath);
    }
});
