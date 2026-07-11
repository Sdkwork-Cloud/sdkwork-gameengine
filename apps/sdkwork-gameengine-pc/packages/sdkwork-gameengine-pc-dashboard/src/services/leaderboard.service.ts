import { getGamesLeaderboardService, type GameLeaderboardItem } from 'sdkwork-gameengine-pc-core';

export interface LeaderboardRow {
  rank: number;
  name: string;
  score: number;
  title: string;
  avatar: string;
  winRate: string;
  trend: string;
  userId: string;
}

export interface LeaderboardPageResult {
  items: LeaderboardRow[];
  total: number;
  page: number;
  pageSize: number;
}

function avatarForUser(userId: string, displayName?: string): string {
  const seed = encodeURIComponent(displayName?.trim() || userId);
  return `https://api.dicebear.com/7.x/initials/svg?seed=${seed}`;
}

export function mapLeaderboardItem(item: GameLeaderboardItem, index: number): LeaderboardRow {
  const rank = item.rankNo ?? index + 1;
  const name = item.displayName?.trim() || item.userId;
  return {
    rank,
    name,
    score: item.score,
    title: 'Player',
    avatar: avatarForUser(item.userId, name),
    winRate: '-',
    trend: '0',
    userId: item.userId,
  };
}

export class LeaderboardService {
  static async listRankings(params?: {
    gameId?: string;
    page?: number;
    pageSize?: number;
  }): Promise<LeaderboardPageResult> {
    const page = await getGamesLeaderboardService().listRankings({
      gameId: params?.gameId,
      page: params?.page ?? 1,
      pageSize: params?.pageSize ?? 20,
    });

    return {
      items: page.items.map((item, index) => mapLeaderboardItem(item, index)),
      total: page.total,
      page: page.page,
      pageSize: page.pageSize,
    };
  }

  static async retrieveMyRanking(params?: { gameId?: string }): Promise<LeaderboardRow | null> {
    const item = await getGamesLeaderboardService().retrieveMyRanking(params);
    if (!item) {
      return null;
    }
    return mapLeaderboardItem(item, 0);
  }
}
