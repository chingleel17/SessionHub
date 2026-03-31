export type SessionInfo = {
    id: string;
    cwd?: string | null;
    summary?: string | null;
    summaryCount?: number | null;
    createdAt?: string | null;
    updatedAt?: string | null;
    sessionDir: string;
    parseError: boolean;
    isArchived: boolean;
    notes?: string | null;
    tags: string[];
    hasPlan: boolean;
};

export type AppSettings = {
    copilotRoot: string;
    terminalPath?: string | null;
    externalEditorPath?: string | null;
    showArchived: boolean;
    pinnedProjects?: string[];
};

export type SessionStats = {
    outputTokens: number;
    interactionCount: number;
    toolCallCount: number;
    durationMinutes: number;
    modelsUsed: string[];
    reasoningCount: number;
    toolBreakdown: Record<string, number>;
    isLive: boolean;
};

export type SettingsSection = "general" | "language" | "icon-style";

export type ProjectGroup = {
    key: string;
    title: string;
    pathLabel: string;
    sessions: SessionInfo[];
    updatedAtLabel: string;
};

export type SortKey = "updatedAt" | "createdAt" | "summaryCount" | "summary";

export type RealtimeStatus = "connecting" | "active" | "error";

export type ConfirmDialogState = {
    title: string;
    message: string;
    actionLabel: string;
    tone: "danger" | "primary";
    onConfirm: () => void;
};

export type EditDialogState = {
    title: string;
    message: string;
    actionLabel: string;
    initialValue: string;
    multiline?: boolean;
    onConfirm: (value: string) => void;
};
