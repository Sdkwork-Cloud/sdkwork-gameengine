import type { LeaderboardEntry } from './leaderboard-entry';

export interface SdkWorkLeaderboardEntryResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
