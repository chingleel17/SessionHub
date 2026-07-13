import { EditDialog } from "session-hub";

const noop = () => {};

export const EditNotesMultiline = () => (
  <EditDialog
    dialog={{
      key: "notes",
      title: "編輯筆記",
      message: "為這個 session 加上備註，之後在列表中可以快速辨識。",
      actionLabel: "儲存",
      initialValue:
        "調查 OpenCode SQLite backend 掃描不穩定的問題：\n- 先確認 WAL 模式下的讀取鎖\n- 比對 session 目錄 mtime 與資料庫索引",
      multiline: true,
      onConfirm: noop,
    }}
    onCancel={noop}
    onConfirm={noop}
  />
);

export const RenameTag = () => (
  <EditDialog
    dialog={{
      key: "tag",
      title: "重新命名標籤",
      message: "更新標籤名稱後，所有使用此標籤的 session 都會一併更新。",
      actionLabel: "重新命名",
      initialValue: "backend",
      onConfirm: noop,
    }}
    onCancel={noop}
    onConfirm={noop}
  />
);

export const WithDangerSecondary = () => (
  <EditDialog
    dialog={{
      key: "notes-existing",
      title: "編輯筆記",
      message: "修改現有筆記內容，或清除整段筆記。",
      actionLabel: "儲存",
      secondaryActionLabel: "清除筆記",
      secondaryActionTone: "danger",
      initialValue: "quota chip 改版已驗證，等 hook 整合完成後再關閉此 session。",
      onConfirm: noop,
      onSecondaryAction: noop,
    }}
    onCancel={noop}
    onConfirm={noop}
  />
);
