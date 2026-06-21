import manifest from '../../sdkwork.app.config.json';

export type SdkworkGameenginePcEnvironment = 'development' | 'test' | 'staging' | 'production';

export type SdkworkGameenginePcConfigProfile = 'dev' | 'test' | 'staging' | 'prod';

export type SdkworkGameenginePcDeploymentMode = 'web';
export type SdkworkGameenginePcRuntimeTarget = 'browser';
export type SdkworkGameenginePcBuildMode = SdkworkGameenginePcEnvironment;

export interface SdkworkGameenginePcAuthRuntimeConfig {
  accessTokenHeader: 'Access-Token';
  authTokenHeader: 'Authorization';
  refreshEnabled: boolean;
  tokenManagerMode: 'appbase-global';
  tokenStorage: 'browser-session';
}

export interface SdkworkGameenginePcI18nRuntimeConfig {
  defaultLocale: string;
  fallbackLocale: string;
  supportedLocales: string[];
}

export interface SdkworkGameenginePcDependencySdkBaseUrls {
  appApiBaseUrl?: string;
  backendApiBaseUrl?: string;
}

export interface SdkworkGameenginePcSdkBaseUrls {
  appApiBaseUrl?: string;
  backendApiBaseUrl?: string;
  dependencySdkBaseUrls?: Record<string, SdkworkGameenginePcDependencySdkBaseUrls>;
  sdkBaseUrl?: string;
}

export interface SdkworkGameenginePcRuntimeConfig {
  appApiBaseUrl: string;
  appDisplayName: string;
  appKey: string;
  auth: SdkworkGameenginePcAuthRuntimeConfig;
  backendApiBaseUrl?: string;
  buildMode: SdkworkGameenginePcBuildMode;
  configProfile: SdkworkGameenginePcConfigProfile;
  deploymentMode: SdkworkGameenginePcDeploymentMode;
  environment: SdkworkGameenginePcEnvironment;
  i18n: SdkworkGameenginePcI18nRuntimeConfig;
  runtimeTarget: SdkworkGameenginePcRuntimeTarget;
  sdkBaseUrl?: string;
  sdkBaseUrls?: SdkworkGameenginePcSdkBaseUrls;
  version: string;
}

const environmentByMode: Record<string, SdkworkGameenginePcEnvironment> = {
  development: 'development',
  dev: 'development',
  production: 'production',
  prod: 'production',
  staging: 'staging',
  test: 'test',
};

const profileByEnvironment: Record<SdkworkGameenginePcEnvironment, SdkworkGameenginePcConfigProfile> = {
  development: 'dev',
  production: 'prod',
  staging: 'staging',
  test: 'test',
};

function envValue(key: string): string | undefined {
  const value = import.meta.env[key];
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function resolveEnvironment(mode: string): SdkworkGameenginePcEnvironment {
  return environmentByMode[mode] ?? 'development';
}

function parseSdkBaseUrls(sdkBaseUrl?: string): SdkworkGameenginePcSdkBaseUrls | undefined {
  const raw = envValue('VITE_SDKWORK_GAMEENGINE_PC_SDK_BASE_URLS_JSON');
  if (raw) {
    try {
      return JSON.parse(raw) as SdkworkGameenginePcSdkBaseUrls;
    } catch {
      return undefined;
    }
  }

  if (!sdkBaseUrl) {
    return undefined;
  }

  const normalizedSdkBaseUrl = sdkBaseUrl.replace(/\/+$/u, '');
  return {
    appApiBaseUrl: `${normalizedSdkBaseUrl}/app/v3/api`,
    backendApiBaseUrl: `${normalizedSdkBaseUrl}/backend/v3/api`,
    dependencySdkBaseUrls: {
      'sdkwork-appbase-app-sdk': {
        appApiBaseUrl: `${normalizedSdkBaseUrl}/app/v3/api`,
      },
      'sdkwork-appbase-backend-sdk': {
        backendApiBaseUrl: `${normalizedSdkBaseUrl}/backend/v3/api`,
      },
    },
    sdkBaseUrl: normalizedSdkBaseUrl,
  };
}

export function resolveSdkworkGameenginePcRuntimeConfig(
  mode = import.meta.env.MODE,
): SdkworkGameenginePcRuntimeConfig {
  const environment = resolveEnvironment(mode);
  const sdkBaseUrl = envValue('VITE_SDKWORK_GAMEENGINE_PC_SDK_BASE_URL');
  const sdkBaseUrls = parseSdkBaseUrls(sdkBaseUrl);
  const defaultApiBase = manifest.runtime.apiBaseUrl.replace(/\/+$/u, '');

  return {
    appApiBaseUrl:
      envValue('VITE_SDKWORK_GAMEENGINE_PC_APP_API_BASE_URL') ??
      sdkBaseUrls?.appApiBaseUrl ??
      `${defaultApiBase}/app/v3/api`,
    appDisplayName: manifest.app.displayName,
    appKey: manifest.app.key,
    auth: {
      accessTokenHeader: 'Access-Token',
      authTokenHeader: 'Authorization',
      refreshEnabled: true,
      tokenManagerMode: 'appbase-global',
      tokenStorage: 'browser-session',
    },
    backendApiBaseUrl:
      envValue('VITE_SDKWORK_GAMEENGINE_PC_BACKEND_API_BASE_URL') ?? sdkBaseUrls?.backendApiBaseUrl,
    buildMode: environment,
    configProfile: profileByEnvironment[environment],
    deploymentMode: 'web',
    environment,
    i18n: {
      defaultLocale: envValue('VITE_SDKWORK_GAMEENGINE_PC_DEFAULT_LOCALE') ?? 'zh-CN',
      fallbackLocale: 'en-US',
      supportedLocales: ['zh-CN', 'en-US'],
    },
    runtimeTarget: 'browser',
    sdkBaseUrl,
    sdkBaseUrls,
    version: manifest.release.currentVersion,
  };
}
