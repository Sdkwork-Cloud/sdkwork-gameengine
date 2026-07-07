import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkBackendConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { GamesApi, createGamesApi } from './api/games';
import { RoomsApi, createRoomsApi } from './api/rooms';

export class SdkworkGameengineBackendClient {
  private httpClient: HttpClient;

  public readonly games: GamesApi;
  public readonly rooms: RoomsApi;

  constructor(config: SdkworkBackendConfig) {
    this.httpClient = createHttpClient(config);
    this.games = createGamesApi(this.httpClient);

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

export function createClient(config: SdkworkBackendConfig): SdkworkGameengineBackendClient {
  return new SdkworkGameengineBackendClient(config);
}

export default SdkworkGameengineBackendClient;
