import { configureGamesCatalogService, createGamesCatalogService } from 'sdkwork-gameengine-pc-core';

import type { SdkworkGameenginePcRuntimeConfig } from './environment';
import type { SdkworkGameenginePcIamRuntime } from './iamRuntime';
import type { SdkworkGameenginePcSdkClientInventory } from './sdkClients';

export interface SdkworkGameenginePcProviders {
  catalogService: ReturnType<typeof createGamesCatalogService>;
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
  return { catalogService };
}
