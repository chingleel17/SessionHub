import { expect, test } from "bun:test";
import { onlineManager } from "@tanstack/react-query";
import { createAppQueryClient } from "../src/utils/createAppQueryClient";
import { withTimeout } from "../src/utils/withTimeout";

test("離線時主視窗可更新本機快取，獨立 overlay 也能首次載入", async () => {
  const wasOnline = onlineManager.isOnline();
  const main = createAppQueryClient();
  const overlay = createAppQueryClient();
  onlineManager.setOnline(false);
  try {
    main.setQueryData(["quota"], "old");
    await expect(withTimeout(main.fetchQuery({
      queryKey: ["quota"], queryFn: async () => "new",
    }), 500, "query paused")).resolves.toBe("new");
    await expect(withTimeout(overlay.fetchQuery({
      queryKey: ["quota"], queryFn: async () => "new",
    }), 500, "query paused")).resolves.toBe("new");
    await expect(withTimeout(main.fetchQuery({
      queryKey: ["project_specs", "project", "active"], queryFn: async () => [],
    }), 500, "query paused")).resolves.toEqual([]);
  } finally {
    main.clear();
    overlay.clear();
    onlineManager.setOnline(wasOnline);
  }
});

test("離線時本機設定 mutation 不會暫停", async () => {
  const wasOnline = onlineManager.isOnline();
  const client = createAppQueryClient();
  onlineManager.setOnline(false);
  try {
    const mutation = client.getMutationCache().build(client, {
      mutationFn: async (value: string) => value,
    });
    await expect(withTimeout(mutation.execute("saved"), 500, "mutation paused"))
      .resolves.toBe("saved");
  } finally {
    client.clear();
    onlineManager.setOnline(wasOnline);
  }
});
