import type { SessionInfo } from "../types";

export function updateSessionMetadataCache(
  sessions: SessionInfo[] | undefined,
  sessionId: string,
  notes: string | null | undefined,
  tags: string[],
): SessionInfo[] | undefined {
  if (!sessions) return undefined;

  return sessions.map((session) =>
    session.id === sessionId ? { ...session, notes: notes ?? null, tags: [...tags] } : session,
  );
}
