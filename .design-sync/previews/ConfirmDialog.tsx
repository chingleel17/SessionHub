import { ConfirmDialog } from "session-hub";

const noop = () => {};

export const DangerDelete = () => (
  <ConfirmDialog
    dialog={{
      title: "刪除 Session",
      message:
        "確定要刪除 session「Refactor status bar quota chip into SVG ring」嗎？此操作會移除本機索引與筆記，且無法復原。",
      actionLabel: "刪除",
      tone: "danger",
      onConfirm: noop,
    }}
    onCancel={noop}
  />
);

export const PrimaryArchive = () => (
  <ConfirmDialog
    dialog={{
      title: "封存 Session",
      message: "封存後此 session 將移至封存清單，之後仍可隨時取消封存。",
      actionLabel: "封存",
      tone: "primary",
      onConfirm: noop,
    }}
    onCancel={noop}
  />
);
