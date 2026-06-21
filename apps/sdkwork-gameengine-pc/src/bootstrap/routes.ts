import { createSdkworkGameenginePcRouteRegistry } from 'sdkwork-gameengine-pc-core';
import type { SdkworkGameenginePcRouteContribution } from 'sdkwork-gameengine-pc-core';

export const SdkworkGameenginePcDashboardRoutes = [
  {
    auth: 'required',
    capability: 'dashboard',
    domain: 'game',
    id: 'app.games.dashboard.home',
    packageName: 'sdkwork-gameengine-pc-shell',
    path: '/app/games/*',
    screen: 'GamesAppShell',
    surface: 'app',
    title: 'Games',
    titleKey: 'games.dashboard.title',
  },
] as const satisfies readonly SdkworkGameenginePcRouteContribution[];

export const SdkworkGameenginePcRoutes = createSdkworkGameenginePcRouteRegistry(SdkworkGameenginePcDashboardRoutes);

export type { SdkworkGameenginePcRouteContribution };
