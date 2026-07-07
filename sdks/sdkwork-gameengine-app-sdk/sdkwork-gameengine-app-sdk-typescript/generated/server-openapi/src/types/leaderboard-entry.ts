export interface LeaderboardEntry {
  id: string;
  gameId: string;
  userId: string;
  displayName?: string;
  score: string;
  rankNo?: number;
  recordedAt: string;
}
