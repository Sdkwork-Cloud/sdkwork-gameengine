import {
  configureGamesCatalogService,
  configureGamesLeaderboardService,
  configureGamesRoomService,
  createGamesCatalogService,
  createGamesLeaderboardService,
  createGamesRoomService,
} from 'sdkwork-gameengine-pc-core';

import type { SdkworkGameenginePcRuntimeConfig } from './environment';
import type { SdkworkGameenginePcIamRuntime } from './iamRuntime';
import type { SdkworkGameenginePcSdkClientInventory } from './sdkClients';

export interface SdkworkGameenginePcProviders {
  catalogService: ReturnType<typeof createGamesCatalogService>;
  leaderboardService: ReturnType<typeof createGamesLeaderboardService>;
  roomService: ReturnType<typeof createGamesRoomService>;
}

export function configureSdkworkGameenginePcProviders(input: {
  config: SdkworkGameenginePcRuntimeConfig;
  iamRuntime: SdkworkGameenginePcIamRuntime;
  sdkClients: SdkworkGameenginePcSdkClientInventory;
}): SdkworkGameenginePcProviders {
  void input.config;
  void input.iamRuntime;
  const catalogService = createGamesCatalogService(input.sdkClients.gamesAppClient);
  configureGamesCatalogService(catalogService);
  const leaderboardService = createGamesLeaderboardService(input.sdkClients.gamesAppClient);
  configureGamesLeaderboardService(leaderboardService);
  const roomService = createGamesRoomService(input.sdkClients.gamesAppClient);
  configureGamesRoomService(roomService);
  return { catalogService, leaderboardService, roomService };
}
