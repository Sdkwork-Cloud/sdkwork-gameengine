import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const shellPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-shell/src/GamesAppShell.tsx',
);
const sidebarPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/components/Sidebar.tsx',
);
const topbarPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/components/Topbar.tsx',
);
const leaderboardPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/pages/Leaderboard.tsx',
);
const gameCenterPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/pages/GameCenter.tsx',
);
const gameServicePath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/services/game.service.ts',
);
const gameTypesPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/types/game.types.ts',
);
const dashboardIndexPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/index.ts',
);
const gameCardPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/GameCenter/GameCard.tsx',
);
const gameBannerPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/GameCenter/GameBanner.tsx',
);
const recentGamesListPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/GameCenter/RecentGamesList.tsx',
);
const liveMatchesGridPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/GameCenter/LiveMatchesGrid.tsx',
);
const commonsIndexPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/index.ts',
);
const createRoomModalPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/components/CreateRoomModal.tsx',
);
const userStorePath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-core/src/store/useUserStore.ts',
);
const appRoutesPath = path.join(root, 'apps/sdkwork-gameengine-pc/src/AppRoutes.tsx');
const authGatePath = path.join(root, 'apps/sdkwork-gameengine-pc/src/AuthGate.tsx');
const i18nIndexPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/index.ts',
);
const shellPackageJsonPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-shell/package.json',
);
const dashboardPagePath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/pages/Dashboard.tsx',
);
const tournamentsPagePath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/pages/Tournaments.tsx',
);
const dashboardComponentsPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/Dashboard',
);
const matchmakingModalPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/components/MatchmakingModal.tsx',
);
const storeModalPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/components/StoreModal.tsx',
);
const leaderboardChallengeModalPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/Leaderboard/ChallengeModal.tsx',
);
const leaderboardArenaModalPath = path.join(
  root,
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-dashboard/src/components/Leaderboard/ArenaModal.tsx',
);
const retiredI18nDictionaries = [
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/en/dashboard.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/en/arena.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/en/quiz.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/en/ringmatch.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/en/store.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/zh/dashboard.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/zh/arena.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/zh/quiz.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/zh/ringmatch.ts',
  'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-i18n/src/locales/zh/store.ts',
];

const forbiddenProductionShellImports = [
  'sdkwork-gameengine-pc-user',
  'sdkwork-gameengine-pc-vip',
  'sdkwork-gameengine-pc-compute',
  'sdkwork-gameengine-pc-mall',
  'sdkwork-gameengine-pc-wallet',
  'sdkwork-gameengine-pc-arena',
  'sdkwork-gameengine-pc-quiz',
  'sdkwork-gameengine-pc-ringmatch',
  'sdkwork-gameengine-pc-claws',
  'sdkwork-gameengine-pc-auth',
  'sdkwork-gameengine-pc-recharge',
];

const forbiddenProductionViewIds = [
  'dashboard',
  'arena',
  'ringmatch',
  'claws',
  'tournaments',
  'quiz',
  'profile',
  'playerProfile',
  'subscription',
  'compute',
  'mall',
  'wallet',
];

const retiredPackageDirs = [
  'sdkwork-gameengine-pc-arena',
  'sdkwork-gameengine-pc-auth',
  'sdkwork-gameengine-pc-claws',
  'sdkwork-gameengine-pc-compute',
  'sdkwork-gameengine-pc-mall',
  'sdkwork-gameengine-pc-quiz',
  'sdkwork-gameengine-pc-recharge',
  'sdkwork-gameengine-pc-ringmatch',
  'sdkwork-gameengine-pc-user',
  'sdkwork-gameengine-pc-vip',
  'sdkwork-gameengine-pc-wallet',
];

test('production PC shell only mounts implemented SDK-backed game surfaces', () => {
  const shellSource = fs.readFileSync(shellPath, 'utf8');

  for (const packageName of forbiddenProductionShellImports) {
    assert.ok(
      !shellSource.includes(packageName),
      `production shell must not import non-production package ${packageName}`,
    );
  }

  for (const viewId of forbiddenProductionViewIds) {
    assert.ok(
      !shellSource.includes(`case "${viewId}"`),
      `production shell must not mount non-production view ${viewId}`,
    );
  }

  assert.ok(
    !shellSource.includes('Dashboard'),
    'production shell must not mount the mock dashboard surface',
  );
  assert.ok(
    shellSource.includes('useState("games")'),
    'production shell must default to the SDK-backed game catalog surface',
  );
});

test('production sidebar only exposes implemented SDK-backed game navigation', () => {
  const sidebarSource = fs.readFileSync(sidebarPath, 'utf8');

  for (const viewId of forbiddenProductionViewIds) {
    assert.ok(
      !sidebarSource.includes(`id: "${viewId}"`),
      `production sidebar must not expose non-production view ${viewId}`,
    );
  }
});

test('production topbar does not expose local ledger or mock store entrypoints', () => {
  const topbarSource = fs.readFileSync(topbarPath, 'utf8');

  for (const forbiddenSnippet of [
    'StoreModal',
    'isStoreModalOpen',
    'profile?.points',
    'profile?.computeTokens',
    'setCurrentView("profile")',
    '<Bell',
    '<Settings',
    '<Search',
    'privileges_subscription',
    'profile?.vipLevel',
    'currentLevel',
    'currentXP',
    'nextLevelXP',
    'xpPercentage',
    'setCurrentView("auth")',
  ]) {
    assert.ok(
      !topbarSource.includes(forbiddenSnippet),
      `production topbar must not expose local ledger/store snippet ${forbiddenSnippet}`,
    );
  }

  assert.ok(
    topbarSource.includes('onLogout'),
    'production topbar logout must delegate to the IAM runtime-owned logout handler',
  );
});

test('production leaderboard is read-only until challenge APIs exist', () => {
  const leaderboardSource = fs.readFileSync(leaderboardPath, 'utf8');

  for (const forbiddenSnippet of [
    'ChallengeModal',
    'ArenaModal',
    'useUserStore',
    'showChallengeModal',
    'showArenaModal',
    'wagerAmount',
    'challenged_alert',
    'arena_success_alert',
    'strongest_ai',
    'team_rankings',
    'leaderboard_tab_coming_soon',
    'activeTab === "ai"',
    'activeTab !== "global"',
  ]) {
    assert.ok(
      !leaderboardSource.includes(forbiddenSnippet),
      `production leaderboard must not expose local challenge snippet ${forbiddenSnippet}`,
    );
  }
});

test('production game center has no simulated matchmaking, local recent list, or client-side page slicing', () => {
  const gameCenterSource = fs.readFileSync(gameCenterPath, 'utf8');
  const gameCardSource = fs.readFileSync(gameCardPath, 'utf8');
  const liveMatchesGridSource = fs.readFileSync(liveMatchesGridPath, 'utf8');
  const gameServiceSource = fs.readFileSync(gameServicePath, 'utf8');
  const gameTypesSource = fs.readFileSync(gameTypesPath, 'utf8');

  for (const forbiddenSnippet of [
    'MatchmakingModal',
    'isMatchmakingOpen',
    'setIsMatchmakingOpen',
    'selectedGame',
    'handleQuickMatch',
    'handlePlayAction',
    'handleChallengeAIAction',
    'getRecentlyPlayed',
    'getFeaturedBanners',
    '<RecentGamesList',
    '<GameBanner',
    'onQuickMatch',
    'onSpectate',
    'onChallengeAI',
    'setCurrentView("quiz")',
    "{ id: 'quiz'",
    'games.slice(',
  ]) {
    assert.ok(
      !gameCenterSource.includes(forbiddenSnippet),
      `production game center must not expose simulated or client-paginated snippet ${forbiddenSnippet}`,
    );
  }

  for (const [fileLabel, source, snippets] of [
    [
      'GameCard',
      gameCardSource,
      ['onPlay', 'onChallengeAI', 'challenge_ai', 'play_now', '<Play', 'rating'],
    ],
    [
      'LiveMatchesGrid',
      liveMatchesGridSource,
      ['onSpectate', 'spectate', '<Play', 'AI vs Human', 'ai_vs_human', 'bottts', 'alt="AI"', '>AI<'],
    ],
  ]) {
    for (const forbiddenSnippet of snippets) {
      assert.ok(
        !source.includes(forbiddenSnippet),
        `${fileLabel} must not expose unimplemented action snippet ${forbiddenSnippet}`,
      );
    }
  }

  assert.ok(
    !gameServiceSource.includes('static async getRecentlyPlayed'),
    'production game service must not expose an empty recently played placeholder',
  );
  assert.ok(
    !gameServiceSource.includes('static async getFeaturedBanners'),
    'production game service must not derive fake featured banners from catalog rows',
  );
  for (const forbiddenSnippet of ['images.unsplash.com', 'rating: 4.5', 'aiDifficulty']) {
    assert.ok(
      !gameServiceSource.includes(forbiddenSnippet),
      `production game service must not synthesize catalog field ${forbiddenSnippet}`,
    );
  }
  assert.ok(
    gameServiceSource.includes('getGamesRoomService().createRoom'),
    'production game service must create rooms through the configured app SDK room service',
  );
  assert.ok(
    !gameTypesSource.includes("| 'ai'"),
    'production live room types must not retain AI player variants without an AI challenge API',
  );
  for (const retiredPath of [gameBannerPath, recentGamesListPath]) {
    assert.ok(
      !fs.existsSync(retiredPath),
      `retired non-production game center component must be removed: ${path.basename(retiredPath)}`,
    );
  }
});

test('production commons exports no mock commerce or simulated matchmaking components', () => {
  const commonsIndexSource = fs.readFileSync(commonsIndexPath, 'utf8');
  const createRoomModalSource = fs.readFileSync(createRoomModalPath, 'utf8');

  for (const forbiddenSnippet of ['MatchmakingModal', 'StoreModal']) {
    assert.ok(
      !commonsIndexSource.includes(forbiddenSnippet),
      `commons package must not export mock-only component ${forbiddenSnippet}`,
    );
  }

  for (const retiredPath of [matchmakingModalPath, storeModalPath]) {
    assert.ok(
      !fs.existsSync(retiredPath),
      `retired commons mock component must be removed: ${path.basename(retiredPath)}`,
    );
  }

  assert.ok(
    createRoomModalSource.includes('onCreateRoom'),
    'create room modal must delegate creation to an injected SDK-backed handler',
  );
  assert.ok(
    createRoomModalSource.includes('await onCreateRoom'),
    'create room modal must await the injected SDK-backed creation handler',
  );

  for (const forbiddenSnippet of ['Handle room creation', 'password', 'set_room_password']) {
    assert.ok(
      !createRoomModalSource.includes(forbiddenSnippet),
      `create room modal must not expose unsupported room creation snippet ${forbiddenSnippet}`,
    );
  }
});

test('production dashboard package only exports mounted SDK-backed pages', () => {
  const dashboardIndexSource = fs.readFileSync(dashboardIndexPath, 'utf8');

  for (const forbiddenSnippet of ['Dashboard', 'Tournaments']) {
    assert.ok(
      !dashboardIndexSource.includes(forbiddenSnippet),
      `dashboard package must not export non-production page ${forbiddenSnippet}`,
    );
  }

  for (const retiredPath of [
    dashboardPagePath,
    tournamentsPagePath,
    dashboardComponentsPath,
    leaderboardChallengeModalPath,
    leaderboardArenaModalPath,
  ]) {
    assert.ok(
      !fs.existsSync(retiredPath),
      `retired dashboard source must be removed: ${path.basename(retiredPath)}`,
    );
  }
});

test('production i18n bundle does not load retired feature dictionaries', () => {
  const i18nSource = fs.readFileSync(i18nIndexPath, 'utf8');

  for (const forbiddenSnippet of ['dashboard', 'arena', 'quiz', 'ringmatch', 'store']) {
    assert.ok(
      !i18nSource.includes(`./locales/en/${forbiddenSnippet}`),
      `production i18n must not import retired English dictionary ${forbiddenSnippet}`,
    );
    assert.ok(
      !i18nSource.includes(`./locales/zh/${forbiddenSnippet}`),
      `production i18n must not import retired Chinese dictionary ${forbiddenSnippet}`,
    );
  }

  for (const dictionaryPath of retiredI18nDictionaries) {
    assert.ok(
      !fs.existsSync(path.join(root, dictionaryPath)),
      `retired i18n dictionary must be removed: ${dictionaryPath}`,
    );
  }
});

test('production shell package no longer depends on retired mock feature packages', () => {
  const shellPackageJson = JSON.parse(fs.readFileSync(shellPackageJsonPath, 'utf8'));
  const dependencies = shellPackageJson.dependencies ?? {};

  for (const packageName of forbiddenProductionShellImports) {
    assert.ok(
      !(packageName in dependencies),
      `shell package must not depend on retired package ${packageName}`,
    );
  }
});

test('retired mock feature package directories are removed from the PC workspace', () => {
  const packagesRoot = path.join(root, 'apps/sdkwork-gameengine-pc/packages');

  for (const packageDir of retiredPackageDirs) {
    assert.ok(
      !fs.existsSync(path.join(packagesRoot, packageDir)),
      `retired mock package directory must be removed: ${packageDir}`,
    );
  }
});

test('user session store is an IAM session mirror, not a local economy ledger', () => {
  const userStoreSource = fs.readFileSync(userStorePath, 'utf8');

  for (const forbiddenSnippet of [
    'points',
    'computeTokens',
    'vipLevel',
    'transactions',
    'addPoints',
    'deductPoints',
    'addComputeTokens',
    'deductComputeTokens',
    'setVipLevel',
    'addExp',
    'Math.random',
  ]) {
    assert.ok(
      !userStoreSource.includes(forbiddenSnippet),
      `user store must not keep local economy ledger snippet ${forbiddenSnippet}`,
    );
  }

  assert.ok(
    userStoreSource.includes('syncFromIamSession'),
    'user store must keep IAM session synchronization as its identity source',
  );
});

test('auth shell synchronizes identity from IAM and delegates logout to IAM runtime', () => {
  const appRoutesSource = fs.readFileSync(appRoutesPath, 'utf8');
  const authGateSource = fs.readFileSync(authGatePath, 'utf8');

  assert.ok(
    appRoutesSource.includes('onLogout={'),
    'app routes must pass an IAM runtime logout handler into the production shell',
  );
  assert.ok(
    appRoutesSource.includes('runtime.iamRuntime.service.auth.sessions.current.delete()'),
    'app routes logout handler must call the IAM runtime current-session delete operation',
  );
  assert.ok(
    authGateSource.includes('syncFromIamSession'),
    'auth gate must synchronize the local identity mirror from the IAM session snapshot',
  );
});
