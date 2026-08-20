import type { LeaderboardEntry } from './leaderboard-entry';

export interface SdkWorkLeaderboardEntryResponse {
  code: 0;
  data: unknown & { item: LeaderboardEntry; };
  /** Server-owned request correlation id. */
  traceId: string;
}
