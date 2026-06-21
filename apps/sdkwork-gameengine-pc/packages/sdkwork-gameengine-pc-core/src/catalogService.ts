import type { GameCatalogItem, GameCatalogPage } from 'sdkwork-games-app-sdk-generated-typescript';
import type { SdkworkGamesAppClient } from 'sdkwork-games-app-sdk-generated-typescript';

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

export function createGamesCatalogService(client: SdkworkGamesAppClient): GamesCatalogService {
  return {
    async listCatalog(params) {
      const result = await client.games.catalog.list(params);
      return result.data as GameCatalogPage;
    },
    async retrieveCatalogItem(gameId) {
      const result = await client.games.catalog.retrieve(gameId);
      return result.data as GameCatalogItem;
    },
  };
}
