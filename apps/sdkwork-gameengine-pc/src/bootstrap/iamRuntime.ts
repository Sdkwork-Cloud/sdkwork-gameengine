import { createClient as createAppbaseAppClient, type SdkworkAppClient } from '@sdkwork/iam-app-sdk';
import {
  createSdkworkAppbasePcAuthRuntime,
  type SdkworkAppbasePcAuthRuntimeComposition,
  type SdkworkAppbasePcAuthRuntimeSdkClient,
} from '@sdkwork/auth-runtime-pc-react';
import type { IamAppContext, IamDeploymentMode, IamEnvironment } from '@sdkwork/iam-contracts';
import type { IamRuntime } from '@sdkwork/iam-runtime';
import { normalizeSdkworkApiBaseUrl } from '@sdkwork/runtime-bootstrap';
import { createClient as createGameengineAppClient } from '@sdkwork/gameengine-app-sdk';

import type { SdkworkGameenginePcRuntimeConfig } from './environment';
import {
  createSdkworkGameenginePcSessionStore,
  SDKWORK_GAMEENGINE_PC_SESSION_STORAGE_KEY,
  type SdkworkGameenginePcSessionSnapshot,
  type SdkworkGameenginePcSessionStore,
} from './sessionStore';
import { createSdkworkGameenginePcSessionTokenManager } from './sessionTokenManager';
import type { SdkworkGameenginePcSdkClientInventory } from './sdkClients';

const APPBASE_APP_SDK_FAMILY_ID = 'sdkwork-iam-app-sdk';
const APP_API_PREFIX = '/app/v3/api';

export type SdkworkGameenginePcIamRuntime = IamRuntime & {
  composition: SdkworkAppbasePcAuthRuntimeComposition;
  session: SdkworkGameenginePcSessionStore;
};

export interface CreateSdkworkGameenginePcIamRuntimeOptions {
  config: SdkworkGameenginePcRuntimeConfig;
  sdkClients: SdkworkGameenginePcSdkClientInventory;
  session?: SdkworkGameenginePcSessionStore;
}

interface GamesIamSessionLike {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
  sessionId?: string;
  context?: IamAppContext;
}

export function createSdkworkGameenginePcIamRuntime(
  options: CreateSdkworkGameenginePcIamRuntimeOptions,
): SdkworkGameenginePcIamRuntime {
  const session = options.session ?? createSdkworkGameenginePcSessionStore(resolveSessionStorage());
  const tokenManager = createSdkworkGameenginePcSessionTokenManager(session);
  const appbaseAppClient = createAppbaseGeneratedAppClient(options.config, tokenManager);
  const composition = createSdkworkAppbasePcAuthRuntime({
    app: {
      appId: options.config.appKey,
      deploymentMode: toIamDeploymentMode(options.config.deploymentMode),
      environment: toIamEnvironment(options.config.environment),
      platform: 'pc',
    },
    baseUrls: {
      appbaseAppApiBaseUrl: resolveAppbaseAppApiBaseUrl(options.config),
    },
    createAppbaseAppClient: () => appbaseAppClient,
    localeProvider: () => options.config.i18n.defaultLocale,
    sdkClients: [options.sdkClients.gamesAppClient] as SdkworkAppbasePcAuthRuntimeSdkClient[],
    sessionBridge: {
      clearSession: () => {
        session.clearSession();
      },
      commitSession: (nextSession) =>
        commitGamesIamRuntimeSession(session, nextSession as GamesIamSessionLike),
      readSession: () => toGamesIamBridgeSession(session.getSnapshot()),
    },
    tokenManager,
  });

  return {
    ...composition.runtime,
    composition,
    session,
  };
}

export function createSdkworkGameenginePcSdkClientsWithTokenManager(
  config: SdkworkGameenginePcRuntimeConfig,
  tokenManager: ReturnType<typeof createSdkworkGameenginePcSessionTokenManager>,
): SdkworkGameenginePcSdkClientInventory {
  const gamesAppClient = createGameengineAppClient({
    authMode: 'dual-token',
    baseUrl: normalizeGeneratedSdkBaseUrl(config.appApiBaseUrl, APP_API_PREFIX),
    platform: 'pc',
    tokenManager,
  });

  gamesAppClient.setTokenManager(tokenManager);

  return {
    appApiBaseUrl: normalizeSdkworkApiBaseUrl(config.appApiBaseUrl, 'app'),
    backendApiBaseUrl: config.backendApiBaseUrl
      ? normalizeSdkworkApiBaseUrl(config.backendApiBaseUrl, 'backend')
      : undefined,
    gamesAppClient,
    sdkFamilies: {
      app: ['sdkwork-gameengine-app-sdk', 'sdkwork-iam-app-sdk'],
    },
  };
}

function createAppbaseGeneratedAppClient(
  config: SdkworkGameenginePcRuntimeConfig,
  tokenManager: ReturnType<typeof createSdkworkGameenginePcSessionTokenManager>,
): SdkworkAppClient {
  return createAppbaseAppClient({
    authMode: 'dual-token',
    baseUrl: normalizeGeneratedSdkBaseUrl(resolveAppbaseAppApiBaseUrl(config), APP_API_PREFIX),
    platform: 'pc',
    tokenManager,
  });
}

function resolveAppbaseAppApiBaseUrl(config: SdkworkGameenginePcRuntimeConfig): string {
  return (
    config.sdkBaseUrls?.dependencySdkBaseUrls?.[APPBASE_APP_SDK_FAMILY_ID]?.appApiBaseUrl ??
    config.appApiBaseUrl
  );
}

function normalizeGeneratedSdkBaseUrl(baseUrl: string, apiPrefix: string): string {
  const normalizedBaseUrl = baseUrl.replace(/\/+$/u, '');
  const normalizedApiPrefix = apiPrefix.replace(/\/+$/u, '');
  if (normalizedBaseUrl.endsWith(normalizedApiPrefix)) {
    return normalizedBaseUrl.slice(0, -normalizedApiPrefix.length) || normalizedBaseUrl;
  }
  return normalizedBaseUrl;
}

function commitGamesIamRuntimeSession(
  session: SdkworkGameenginePcSessionStore,
  iamSession: GamesIamSessionLike,
): GamesIamSessionLike | undefined {
  const nextSession: SdkworkGameenginePcSessionSnapshot = {
    ...session.getSnapshot(),
    accessToken: iamSession.accessToken,
    authToken: iamSession.authToken,
    refreshToken: iamSession.refreshToken,
    sessionId: iamSession.sessionId ?? iamSession.context?.sessionId,
    context: iamSession.context
      ? {
          tenantId: iamSession.context.tenantId,
          userId: iamSession.context.userId,
          organizationId: iamSession.context.organizationId,
          sessionId: iamSession.context.sessionId,
          appId: iamSession.context.appId,
          environment: iamSession.context.environment,
          deploymentMode: iamSession.context.deploymentMode,
        }
      : undefined,
  };

  if (!nextSession.context) {
    delete nextSession.context;
  }

  session.setSession(nextSession);
  return toGamesIamBridgeSession(session.getSnapshot()) ?? undefined;
}

function toGamesIamBridgeSession(
  snapshot: SdkworkGameenginePcSessionSnapshot,
): GamesIamSessionLike | null {
  if (!snapshot.authToken && !snapshot.accessToken && !snapshot.refreshToken) {
    return null;
  }

  return {
    ...(snapshot.accessToken ? { accessToken: snapshot.accessToken } : {}),
    ...(snapshot.authToken ? { authToken: snapshot.authToken } : {}),
    ...(snapshot.refreshToken ? { refreshToken: snapshot.refreshToken } : {}),
    ...(snapshot.sessionId ? { sessionId: snapshot.sessionId } : {}),
    ...(snapshot.context?.tenantId && snapshot.context.userId
      ? {
          context: {
            tenantId: snapshot.context.tenantId,
            userId: snapshot.context.userId,
            organizationId: snapshot.context.organizationId,
            sessionId: snapshot.context.sessionId,
            appId: snapshot.context.appId,
            environment: snapshot.context.environment,
            deploymentMode: snapshot.context.deploymentMode,
          } as IamAppContext,
        }
      : {}),
  };
}

function resolveSessionStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }
  migrateLegacySessionStorage(SDKWORK_GAMEENGINE_PC_SESSION_STORAGE_KEY);
  return window.localStorage;
}

function migrateLegacySessionStorage(storageKey: string): void {
  const legacySession = window.sessionStorage.getItem(storageKey);
  if (legacySession && !window.localStorage.getItem(storageKey)) {
    window.localStorage.setItem(storageKey, legacySession);
  }
  if (legacySession) {
    window.sessionStorage.removeItem(storageKey);
  }
}

function toIamDeploymentMode(value: SdkworkGameenginePcRuntimeConfig['deploymentMode']): IamDeploymentMode {
  return value === 'web' ? 'saas' : value;
}

function toIamEnvironment(value: SdkworkGameenginePcRuntimeConfig['environment']): IamEnvironment {
  if (value === 'development') {
    return 'dev';
  }
  if (value === 'production') {
    return 'prod';
  }
  if (value === 'staging') {
    return 'test';
  }
  return 'test';
}
