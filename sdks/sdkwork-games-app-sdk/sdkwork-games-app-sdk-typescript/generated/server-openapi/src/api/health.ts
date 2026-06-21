import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { GamesHealthResponse } from '../types';


export class HealthGamesReadyApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async check(): Promise<GamesHealthResponse> {
    return this.client.get<GamesHealthResponse>(appApiPath(`/system/ready`));
  }
}

export class HealthGamesHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async check(): Promise<GamesHealthResponse> {
    return this.client.get<GamesHealthResponse>(appApiPath(`/system/health`));
  }
}

export class HealthGamesApi {
  private client: HttpClient;
  public readonly health: HealthGamesHealthApi;
  public readonly ready: HealthGamesReadyApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.health = new HealthGamesHealthApi(client);
    this.ready = new HealthGamesReadyApi(client);
  }

}

export class HealthApi {
  private client: HttpClient;
  public readonly games: HealthGamesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.games = new HealthGamesApi(client);
  }

}

export function createHealthApi(client: HttpClient): HealthApi {
  return new HealthApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
