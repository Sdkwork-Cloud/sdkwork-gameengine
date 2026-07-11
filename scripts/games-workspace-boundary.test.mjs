import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isBlank, slugify } from '@sdkwork/utils';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('workspace exposes sdkwork.app.config.json with game domain', () => {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, 'sdkwork.app.config.json'), 'utf8'),
  );
  assert.equal(manifest.app.domain, 'game');
  assert.ok(!isBlank(manifest.sdk.appSdkFamily));
});

test('route manifests declare WebRequestContext on protected routes', () => {
  const manifestPath = path.join(
    root,
    'sdks/_route-manifests/app-api/sdkwork-routes-catalog-app-api.route-manifest.json',
  );
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const protectedRoute = manifest.routes.find((route) => route.path.includes('/games'));
  assert.equal(protectedRoute.requestContext, 'WebRequestContext');
});

test('utils slugify normalizes game codes', () => {
  assert.equal(slugify('Catalog Game 2026'), 'catalog-game-2026');
});

test('production build uses the Rust release profile', () => {
  const dispatcher = fs.readFileSync(path.join(root, 'scripts/sdkwork-command.mjs'), 'utf8');
  const buildCase = dispatcher.match(/case 'build':[\s\S]*?break;/u)?.[0] ?? '';

  assert.match(buildCase, /\bcargo build --workspace --release\b/u);
});

test('dispatcher rejects retired deployment profile values before running workflows', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/sdkwork-command.mjs', 'build', '--deployment-profile', 'cloud-hosted'],
    {
      cwd: root,
      encoding: 'utf8',
      windowsHide: true,
    },
  );

  assert.notEqual(result.status, 0);
  assert.match(`${result.stdout}\n${result.stderr}`, /unsupported deployment profile/i);
  assert.doesNotMatch(`${result.stdout}\n${result.stderr}`, /tsc --noEmit|Compiling/u);
});

test('PC app TypeScript pins React ambient types to the app root', () => {
  const tsconfig = JSON.parse(
    fs.readFileSync(path.join(root, 'apps/sdkwork-gameengine-pc/tsconfig.json'), 'utf8'),
  );
  const paths = tsconfig.compilerOptions?.paths ?? {};

  assert.deepEqual(paths.react, ['./node_modules/@types/react']);
  assert.deepEqual(paths['react/jsx-runtime'], ['./node_modules/@types/react/jsx-runtime.d.ts']);
  assert.deepEqual(paths['react/jsx-dev-runtime'], ['./node_modules/@types/react/jsx-dev-runtime.d.ts']);
  assert.deepEqual(paths['react-dom'], ['./node_modules/@types/react-dom']);
});

test('PC app Vite runtime deduplicates React peer dependencies', () => {
  const viteConfig = fs.readFileSync(
    path.join(root, 'apps/sdkwork-gameengine-pc/vite.config.ts'),
    'utf8',
  );

  assert.match(viteConfig, /dedupe:\s*\[[^\]]*['"]react['"][^\]]*['"]react-dom['"][^\]]*\]/u);
});
