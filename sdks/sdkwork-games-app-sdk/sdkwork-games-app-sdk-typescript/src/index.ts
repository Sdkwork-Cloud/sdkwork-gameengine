import {
  createClient as createGeneratedGamesAppClient,
  SdkworkAppClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common';

export { SdkworkAppClient, createGeneratedGamesAppClient };
export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export type SdkworkGamesAppClient = SdkworkAppClient;

export function createGamesAppClient(config: SdkworkAppConfig): SdkworkGamesAppClient {
  return createGeneratedGamesAppClient(config);
}

export function createClient(config: SdkworkAppConfig): SdkworkGamesAppClient {
  return createGamesAppClient(config);
}
