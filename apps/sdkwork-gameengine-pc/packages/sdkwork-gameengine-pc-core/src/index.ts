export type SdkworkGameenginePcSdkSurface = 'app';

export type SdkworkGameenginePcCredentialScope = 'global-session';

export type SdkworkGameenginePcRouteSurface = 'app';

export interface SdkworkGameenginePcRouteContribution {
  readonly auth: 'public' | 'required';
  readonly capability: string;
  readonly domain: 'game';
  readonly id: string;
  readonly packageName: string;
  readonly path: string;
  readonly screen: string;
  readonly surface: SdkworkGameenginePcRouteSurface;
  readonly title: string;
  readonly titleKey: string;
}

export interface SdkworkGameenginePcSdkFamilyInventoryItem {
  readonly authority: string;
  readonly family: string;
  readonly generationInputSpec: string;
  readonly generatedPackageName?: string;
  readonly surface: SdkworkGameenginePcSdkSurface;
  readonly tokenManagerScope: SdkworkGameenginePcCredentialScope;
}

export const SdkworkGameenginePcRuntimeIdentity = {
  appKey: 'sdkwork-gameengine-pc',
  architecture: 'pc-react',
  domain: 'game',
  runtimeFamily: 'web',
} as const;

export const SdkworkGameenginePcAppSdkFamilies = [
  {
    authority: 'sdkwork-games-app-api',
    family: 'sdkwork-games-app-sdk',
    generationInputSpec: 'apis/app-api/game/games-app-api.openapi.json',
    generatedPackageName: 'sdkwork-games-app-sdk-generated-typescript',
    surface: 'app',
    tokenManagerScope: 'global-session',
  },
  {
    authority: 'sdkwork-appbase-app-api',
    family: 'sdkwork-appbase-app-sdk',
    generationInputSpec:
      '../sdkwork-appbase/sdks/sdkwork-appbase-app-sdk/openapi/sdkwork-appbase-app-api.openapi.yaml',
    generatedPackageName: '@sdkwork/appbase-app-sdk',
    surface: 'app',
    tokenManagerScope: 'global-session',
  },
] as const satisfies readonly SdkworkGameenginePcSdkFamilyInventoryItem[];

export function listSdkworkGameenginePcAppSdkFamilies(): readonly SdkworkGameenginePcSdkFamilyInventoryItem[] {
  return SdkworkGameenginePcAppSdkFamilies;
}

export function createSdkworkGameenginePcRouteRegistry(
  ...routeGroups: readonly (readonly SdkworkGameenginePcRouteContribution[])[]
): readonly SdkworkGameenginePcRouteContribution[] {
  return routeGroups.flat();
}

export * from './store/configStore';
export * from './store/useUserStore';
export * from './catalogService';
