import { listSdkworkGameenginePcAppSdkFamilies } from 'sdkwork-gameengine-pc-core/composition';
import type { SdkworkGameengineAppClient } from '@sdkwork/gameengine-app-sdk';

import type { SdkworkGameenginePcRuntimeConfig } from './environment';

export interface SdkworkGameenginePcSdkClientInventory {
  appApiBaseUrl: string;
  backendApiBaseUrl?: string;
  gamesAppClient: SdkworkGameengineAppClient & { setTokenManager(manager: unknown): unknown };
  sdkFamilies: {
    app: string[];
  };
}

export function listSdkworkGameenginePcRegisteredSdkFamilies(
  config: SdkworkGameenginePcRuntimeConfig,
): SdkworkGameenginePcSdkClientInventory['sdkFamilies'] {
  void config;
  return {
    app: listSdkworkGameenginePcAppSdkFamilies()
      .filter((sdkFamily) => sdkFamily.surface === 'app')
      .map((sdkFamily) => sdkFamily.family),
  };
}
