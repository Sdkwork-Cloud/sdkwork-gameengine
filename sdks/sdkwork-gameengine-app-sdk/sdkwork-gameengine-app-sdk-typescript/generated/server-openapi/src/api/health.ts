import { appApiPath } from './paths';
import type { ApiRequestOptions, HttpClient } from '../http/client';

import type { GamesHealthResponse } from '../types';


export class HealthGamesReadyApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<GamesHealthResponse> {
    return this.client.request<GamesHealthResponse>(appApiPath(`/games/ready`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class HealthGamesHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(requestOptions?: ApiRequestOptions): Promise<GamesHealthResponse> {
    return this.client.request<GamesHealthResponse>(appApiPath(`/games/health`), { ...(requestOptions?.signal !== undefined ? { signal: requestOptions.signal } : {}), ...(requestOptions?.timeout !== undefined ? { timeout: requestOptions.timeout } : {}), method: 'GET' as any, sdkworkUnwrapKind: 'item' });
  }
}

export class HealthGamesApi {
  public readonly health: HealthGamesHealthApi;
  public readonly ready: HealthGamesReadyApi;

  constructor(client: HttpClient) {
    this.health = new HealthGamesHealthApi(client);
    this.ready = new HealthGamesReadyApi(client);
  }

}

export class HealthApi {
  public readonly games: HealthGamesApi;

  constructor(client: HttpClient) {
    this.games = new HealthGamesApi(client);
  }

}

export function createHealthApi(client: HttpClient): HealthApi {
  return new HealthApi(client);
}
