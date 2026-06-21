import {
  resolveSdkworkGameenginePcRuntimeConfig,
  type SdkworkGameenginePcRuntimeConfig,
} from './environment';
import { configureSdkworkGameenginePcProviders } from './gamesProviders';
import {
  createSdkworkGameenginePcIamRuntime,
  createSdkworkGameenginePcSdkClientsWithTokenManager,
  type SdkworkGameenginePcIamRuntime,
} from './iamRuntime';
import { SdkworkGameenginePcRoutes } from './routes';
import {
  createSdkworkGameenginePcSessionStore,
  type SdkworkGameenginePcSessionStore,
} from './sessionStore';
import { createSdkworkGameenginePcSessionTokenManager } from './sessionTokenManager';
import type { SdkworkGameenginePcSdkClientInventory } from './sdkClients';
import type { SdkworkGameenginePcProviders } from './gamesProviders';

export interface SdkworkGameenginePcRuntime {
  catalogService: SdkworkGameenginePcProviders['catalogService'];
  config: SdkworkGameenginePcRuntimeConfig;
  iamRuntime: SdkworkGameenginePcIamRuntime;
  routes: typeof SdkworkGameenginePcRoutes;
  sdkClients: SdkworkGameenginePcSdkClientInventory;
  session: SdkworkGameenginePcSessionStore;
}

export function createSdkworkGameenginePcRuntime(): SdkworkGameenginePcRuntime {
  const config = resolveSdkworkGameenginePcRuntimeConfig();
  const session = createSdkworkGameenginePcSessionStore(
    typeof window === 'undefined' ? undefined : window.sessionStorage,
  );
  const tokenManager = createSdkworkGameenginePcSessionTokenManager(session);
  const sdkClients = createSdkworkGameenginePcSdkClientsWithTokenManager(config, tokenManager);
  const iamRuntime = createSdkworkGameenginePcIamRuntime({
    config,
    sdkClients,
    session,
  });
  const { catalogService } = configureSdkworkGameenginePcProviders({
    config,
    iamRuntime,
    sdkClients,
  });

  return {
    catalogService,
    config,
    iamRuntime,
    routes: SdkworkGameenginePcRoutes,
    sdkClients,
    session,
  };
}
