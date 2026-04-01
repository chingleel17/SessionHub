export type SessionInfo = {
    id: string;
    provider: string;
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
    hasEvents: boolean;
};

export type AppSettings = {
    copilotRoot: string;
    opencodeRoot: string;
    terminalPath?: string | null;
    externalEditorPath?: string | null;
    showArchived: boolean;
    pinnedProjects?: string[];
    enabledProviders: string[];
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

// Sisyphus (.sisyphus) 相關型別

export type SisyphusBoulder = {
    activePlan: string | null;
    planName: string | null;
    agent: string | null;
    sessionIds: string[];
    startedAt: string | null;
};

export type SisyphusPlan = {
    name: string;
    path: string;
    title: string | null;
    tldr: string | null;
    isActive: boolean;
};

export type SisyphusNotepad = {
    name: string;
    hasIssues: boolean;
    hasLearnings: boolean;
};

export type SisyphusData = {
    activePlan: SisyphusBoulder | null;
    plans: SisyphusPlan[];
    notepads: SisyphusNotepad[];
    evidenceFiles: string[];
    draftFiles: string[];
};

// OpenSpec 相關型別

export type OpenSpecChange = {
    name: string;
    hasProposal: boolean;
    hasDesign: boolean;
    hasTasks: boolean;
    specsCount: number;
};

export type OpenSpecSpec = {
    name: string;
    path: string;
};

export type OpenSpecData = {
    schema: string | null;
    activeChanges: OpenSpecChange[];
    archivedChanges: OpenSpecChange[];
    specs: OpenSpecSpec[];
};
