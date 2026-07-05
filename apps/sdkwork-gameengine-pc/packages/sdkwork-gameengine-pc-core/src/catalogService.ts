import type { GameCatalogItem, GameCatalogPage } from '@sdkwork/games-app-sdk';
import type { SdkworkGamesAppClient } from '@sdkwork/games-app-sdk';

export type { GameCatalogItem, GameCatalogPage };

export interface GamesCatalogListParams {
  page?: number;
  pageSize?: number;
  status?: string;
}

export interface GamesCatalogService {
  listCatalog(params?: GamesCatalogListParams): Promise<GameCatalogPage>;
  retrieveCatalogItem(gameId: string): Promise<GameCatalogItem>;
}

let configuredCatalogService: GamesCatalogService | null = null;

export function configureGamesCatalogService(service: GamesCatalogService): void {
  configuredCatalogService = service;
}

export function getGamesCatalogService(): GamesCatalogService {
  if (!configuredCatalogService) {
    throw new Error('Games catalog service is not configured. Bootstrap sdkwork-gameengine-pc runtime first.');
  }
  return configuredCatalogService;
}

function mapPageData(page: import('@sdkwork/games-app-sdk').SdkWorkPageData): GameCatalogPage {
  return {
    items: page.items as unknown as GameCatalogItem[],
    total: Number(page.pageInfo.totalItems ?? page.items.length),
    page: page.pageInfo.page ?? 1,
    pageSize: page.pageInfo.pageSize ?? 20,
  };
}

export function createGamesCatalogService(client: SdkworkGamesAppClient): GamesCatalogService {
  return {
    async listCatalog(params) {
      const page = await client.games.catalog.list(params);
      return mapPageData(page);
    },
    async retrieveCatalogItem(gameId) {
      const item = await client.games.catalog.retrieve(gameId);
      return item as unknown as GameCatalogItem;
    },
  };
}
