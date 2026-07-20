import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

test('OpenAPI export tooling uses authored SDKWork v3 contracts as source of truth', () => {
  const script = fs.readFileSync(path.join(root, 'tools/games_openapi_export.mjs'), 'utf8');

  assert.doesNotMatch(script, /\bGamesApiResult\b/);
  assert.doesNotMatch(script, /\bmessage\b/);
  assert.match(script, /apis\/app-api\/game\/games-app-api\.openapi\.json/);
  assert.match(script, /generated\/openapi\/games-app-api\.openapi\.json/);
});

test('OpenAPI export check detects stale generated contracts without rewriting them', () => {
  const generatedPath = path.join(root, 'generated/openapi/games-app-api.openapi.json');
  const original = fs.readFileSync(generatedPath, 'utf8');
  const stale = `${JSON.stringify({ ...JSON.parse(original), 'x-sdkwork-stale-test': true }, null, 2)}\n`;

  fs.writeFileSync(generatedPath, stale);
  try {
    const result = spawnSync(process.execPath, ['tools/games_openapi_export.mjs', '--check'], {
      cwd: root,
      encoding: 'utf8',
      windowsHide: true,
    });

    assert.notEqual(result.status, 0, 'stale generated OpenAPI export should fail --check');
    assert.match(`${result.stdout}\n${result.stderr}`, /generated\/openapi\/games-app-api\.openapi\.json/);
    assert.equal(fs.readFileSync(generatedPath, 'utf8'), stale, '--check must not rewrite stale output');
  } finally {
    fs.writeFileSync(generatedPath, original);
  }
});

test('api check verifies generated OpenAPI exports before SDK alignment', () => {
  const packageJson = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));

  assert.match(packageJson.scripts['api:check'], /node tools\/games_openapi_export\.mjs --check/);
});

test('authored production sources do not expose legacy per-domain ApiResult names', () => {
  const scanRoots = ['apis', 'apps', 'crates', 'database', 'docs', 'scripts', 'tools', 'sdks'];
  const offenders = [];

  for (const scanRoot of scanRoots) {
    const absoluteRoot = path.join(root, scanRoot);
    if (!fs.existsSync(absoluteRoot)) {
      continue;
    }
    for (const filePath of walk(absoluteRoot)) {
      const source = fs.readFileSync(filePath, 'utf8');
      if (/\bGamesApiResult\b/.test(source)) {
        offenders.push(path.relative(root, filePath).replaceAll(path.sep, '/'));
      }
    }
  }

  assert.deepEqual(offenders, []);
});

test('authored production sources do not construct legacy response envelopes', () => {
  const scanRoots = ['apis', 'apps', 'crates', 'scripts', 'tools', 'sdks'];
  const offenders = [];
  const legacyPatterns = [
    /\bProblemDetailsPayload\b/,
    /"code"\s*:\s*"ok"/,
    /"message"\s*:\s*"success"/,
    /\bcode:\s*Option\s*<\s*String\s*>/,
    /\bok_envelope\b/,
  ];

  for (const scanRoot of scanRoots) {
    const absoluteRoot = path.join(root, scanRoot);
    if (!fs.existsSync(absoluteRoot)) {
      continue;
    }
    for (const filePath of walk(absoluteRoot)) {
      const source = fs.readFileSync(filePath, 'utf8');
      if (legacyPatterns.some((pattern) => pattern.test(source))) {
        offenders.push(path.relative(root, filePath).replaceAll(path.sep, '/'));
      }
    }
  }

  assert.deepEqual(offenders, []);
});

test('gateway runtime bootstrap uses database repositories without demo memory seed paths', () => {
  const gatewaySrc = path.join(root, 'crates/sdkwork-api-gameengine-standalone-gateway/src');
  const offenders = [];
  const forbiddenPatterns = [
    /\bGAMES_REPOSITORY_MODE\b/,
    /\bdemo_seed_/,
    /\bwith_seed\s*\(/,
    /\bInMemory[A-Za-z]+Repository\b/,
    /\bGameCatalogRepositoryAdapter\b/,
  ];

  for (const filePath of walk(gatewaySrc)) {
    const source = fs.readFileSync(filePath, 'utf8');
    if (forbiddenPatterns.some((pattern) => pattern.test(source))) {
      offenders.push(path.relative(root, filePath).replaceAll(path.sep, '/'));
    }
  }

  assert.deepEqual(offenders, []);
});

test('room app-api contract exposes complete player room lifecycle operations', () => {
  const manifestPath = path.join(
    root,
    'sdks/_route-manifests/app-api/sdkwork-routes-room-app-api.route-manifest.json',
  );
  const openApiPath = path.join(root, 'apis/app-api/game/games-app-api.openapi.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const openApi = JSON.parse(fs.readFileSync(openApiPath, 'utf8'));
  const expected = [
    ['GET', '/app/v3/api/games/rooms', 'games.rooms.list'],
    ['POST', '/app/v3/api/games/rooms', 'games.rooms.create'],
    ['GET', '/app/v3/api/games/rooms/{roomId}', 'games.rooms.retrieve'],
    ['GET', '/app/v3/api/games/rooms/{roomId}/seats', 'games.rooms.seats.list'],
    ['POST', '/app/v3/api/games/rooms/{roomId}/join', 'games.rooms.join'],
    ['POST', '/app/v3/api/games/rooms/{roomId}/leave', 'games.rooms.leave'],
    ['POST', '/app/v3/api/games/rooms/{roomId}/ready', 'games.rooms.ready'],
    ['POST', '/app/v3/api/games/rooms/{roomId}/start', 'games.rooms.start'],
    ['POST', '/app/v3/api/games/rooms/{roomId}/close', 'games.rooms.close'],
  ];

  for (const [method, routePath, operationId] of expected) {
    assert.ok(
      manifest.routes.some(
        (route) => route.method === method && route.path === routePath && route.operationId === operationId,
      ),
      `manifest missing ${method} ${routePath} ${operationId}`,
    );
    assert.equal(
      openApi.paths[routePath]?.[method.toLowerCase()]?.operationId,
      operationId,
      `OpenAPI missing ${method} ${routePath} ${operationId}`,
    );
    assert.equal(
      openApi.paths[routePath]?.[method.toLowerCase()]?.['x-sdkwork-request-context'],
      'WebRequestContext',
      `OpenAPI missing request context for ${operationId}`,
    );
    assert.equal(
      openApi.paths[routePath]?.[method.toLowerCase()]?.['x-sdkwork-api-surface'],
      'app-api',
      `OpenAPI missing app-api surface for ${operationId}`,
    );
  }
});

test('room backend-api contract exposes monitoring and force close operations', () => {
  const manifestPath = path.join(
    root,
    'sdks/_route-manifests/backend-api/sdkwork-routes-room-backend-api.route-manifest.json',
  );
  const openApiPath = path.join(root, 'apis/backend-api/game/games-backend-api.openapi.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const openApi = JSON.parse(fs.readFileSync(openApiPath, 'utf8'));
  const expected = [
    ['GET', '/backend/v3/api/games/rooms', 'backend.games.rooms.list'],
    ['GET', '/backend/v3/api/games/rooms/{roomId}', 'backend.games.rooms.retrieve'],
    ['GET', '/backend/v3/api/games/rooms/{roomId}/seats', 'backend.games.rooms.seats.list'],
    ['POST', '/backend/v3/api/games/rooms/{roomId}/force_close', 'backend.games.rooms.forceClose'],
  ];

  for (const [method, routePath, operationId] of expected) {
    assert.ok(
      manifest.routes.some(
        (route) => route.method === method && route.path === routePath && route.operationId === operationId,
      ),
      `manifest missing ${method} ${routePath} ${operationId}`,
    );
    assert.equal(
      openApi.paths[routePath]?.[method.toLowerCase()]?.operationId,
      operationId,
      `OpenAPI missing ${method} ${routePath} ${operationId}`,
    );
    assert.equal(
      openApi.paths[routePath]?.[method.toLowerCase()]?.['x-sdkwork-request-context'],
      'WebRequestContext',
      `OpenAPI missing request context for ${operationId}`,
    );
    assert.equal(
      openApi.paths[routePath]?.[method.toLowerCase()]?.['x-sdkwork-api-surface'],
      'backend-api',
      `OpenAPI missing backend-api surface for ${operationId}`,
    );
  }
});

function* walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === 'target') {
      continue;
    }
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      yield* walk(entryPath);
      continue;
    }
    if (entry.isFile() && isTextFile(entry.name)) {
      yield entryPath;
    }
  }
}

function isTextFile(fileName) {
  return /\.(?:json|md|mjs|js|ts|tsx|rs|toml|yaml|yml|sql)$/i.test(fileName);
}
