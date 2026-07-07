import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkAppConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { HealthApi, createHealthApi } from './api/health';
import { GamesApi, createGamesApi } from './api/games';
import { LeaderboardApi, createLeaderboardApi } from './api/leaderboard';
import { RoomsApi, createRoomsApi } from './api/rooms';

export class SdkworkGameengineAppClient {
  private httpClient: HttpClient;

  public readonly health: HealthApi;
  public readonly games: GamesApi;
  public readonly leaderboard: LeaderboardApi;
  public readonly rooms: RoomsApi;

  constructor(config: SdkworkAppConfig) {
    this.httpClient = createHttpClient(config);
    this.health = createHealthApi(this.httpClient);

    this.games = createGamesApi(this.httpClient);

    this.leaderboard = createLeaderboardApi(this.httpClient);

    this.rooms = createRoomsApi(this.httpClient);
  }
  setAuthToken(token: string): this {
    this.httpClient.setAuthToken(token);
    return this;
  }

  setAccessToken(token: string): this {
    this.httpClient.setAccessToken(token);
    return this;
  }

  setTokenManager(manager: AuthTokenManager): this {
    this.httpClient.setTokenManager(manager);
    return this;
  }

  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkAppConfig): SdkworkGameengineAppClient {
  return new SdkworkGameengineAppClient(config);
}

export default SdkworkGameengineAppClient;
