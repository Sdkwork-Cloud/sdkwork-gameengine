import { SdkWorkResultCode } from '@sdkwork/utils';
import type { SdkWorkPageData } from '@sdkwork/gameengine-app-sdk';
import type { SdkworkGameengineAppClient } from '@sdkwork/gameengine-app-sdk';

export interface GameLeaderboardItem {
  id: string;
  gameId: string;
  userId: string;
  displayName?: string;
  score: number;
  rankNo?: number;
  recordedAt: string;
}

export interface GameLeaderboardPage {
  items: GameLeaderboardItem[];
  total: number;
  page: number;
  pageSize: number;
}

export interface GamesLeaderboardListParams {
  gameId?: string;
  page?: number;
  pageSize?: number;
}

export interface GamesLeaderboardService {
  listRankings(params?: GamesLeaderboardListParams): Promise<GameLeaderboardPage>;
  retrieveMyRanking(params?: { gameId?: string }): Promise<GameLeaderboardItem | null>;
}

let configuredLeaderboardService: GamesLeaderboardService | null = null;

export function configureGamesLeaderboardService(service: GamesLeaderboardService): void {
  configuredLeaderboardService = service;
}

export function getGamesLeaderboardService(): GamesLeaderboardService {
  if (!configuredLeaderboardService) {
    throw new Error(
      'Games leaderboard service is not configured. Bootstrap sdkwork-gameengine-pc runtime first.',
    );
  }
  return configuredLeaderboardService;
}

function mapPageData(page: SdkWorkPageData): GameLeaderboardPage {
  return {
    items: page.items as unknown as GameLeaderboardItem[],
    total: Number(page.pageInfo.totalItems ?? page.items.length),
    page: page.pageInfo.page ?? 1,
    pageSize: page.pageInfo.pageSize ?? 20,
  };
}

function readErrorRecord(error: unknown): Record<string, unknown> | null {
  return error && typeof error === 'object' ? (error as Record<string, unknown>) : null;
}

function isNotFoundError(error: unknown): boolean {
  const record = readErrorRecord(error);
  if (!record) {
    return false;
  }

  return (
    record.code === SdkWorkResultCode.NOT_FOUND
    || record.code === 'NOT_FOUND'
    || record.httpStatus === 404
    || record.status === 404
    || record.statusCode === 404
  );
}

export function createGamesLeaderboardService(client: SdkworkGameengineAppClient): GamesLeaderboardService {
  return {
    async listRankings(params) {
      const page = await client.leaderboard.games.leaderboard.list(params);
      return mapPageData(page);
    },
    async retrieveMyRanking(params) {
      try {
        const item = await client.leaderboard.games.leaderboard.me.retrieve(params);
        return item as unknown as GameLeaderboardItem;
      } catch (error) {
        if (isNotFoundError(error)) {
          return null;
        }
        throw error;
      }
    },
  };
}
