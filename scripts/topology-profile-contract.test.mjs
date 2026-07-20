import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function readJson(path) {
  return JSON.parse(readFileSync(resolve(repoRoot, path), 'utf8'));
}

function readProfileEnv(path) {
  const source = readFileSync(resolve(repoRoot, path), 'utf8');
  const values = {};
  for (const rawLine of source.split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;
    const [key, ...valueParts] = line.split('=');
    values[key] = valueParts.join('=');
  }
  return { source, values };
}

test('topology env keys use the games application code', () => {
  const topology = readJson('specs/topology.spec.json');
  const serialized = JSON.stringify(topology);

  for (const value of Object.values(topology.envKeys)) {
    if (value.startsWith('SDKWORK_') && value !== 'SDKWORK_API_CLOUD_GATEWAY_BIND' && value !== 'SDKWORK_API_CLOUD_GATEWAY_CONFIG') {
      assert.ok(
        value.startsWith('SDKWORK_GAMES_'),
        `${value} must use SDKWORK_GAMES_* because the application code is games`,
      );
    }
    if (value.startsWith('VITE_')) {
      assert.ok(
        value.startsWith('VITE_SDKWORK_GAMES_'),
        `${value} must use VITE_SDKWORK_GAMES_* because the application code is games`,
      );
    }
  }

  assert.ok(!serialized.includes('SDKWORK_GAMEENGINE_'));
  assert.ok(!serialized.includes('VITE_SDKWORK_GAMEENGINE_'));
});

test('topology profiles use SDKWork v4 two-segment profile ids', () => {
  const topology = readJson('specs/topology.spec.json');
  const expectedProfileFiles = {
    'standalone.development': 'configs/topology/standalone.development.env',
    'standalone.production': 'configs/topology/standalone.production.env',
    'cloud.development': 'configs/topology/cloud.development.env',
    'cloud.production': 'configs/topology/cloud.production.env',
  };

  assert.equal(topology.profilePattern, '{deploymentProfile}.{environment}.env');
  assert.ok(!('serviceLayout' in topology.vocabulary), 'service layout must not be a public topology axis');
  assert.ok(!('serviceLayout' in topology.envKeys), 'service layout must not have a public env key');
  assert.deepEqual(topology.profileFiles, expectedProfileFiles);

  for (const [profileId, profilePath] of Object.entries(topology.profileFiles)) {
    const segments = profileId.split('.');
    assert.equal(segments.length, 2, `${profileId} must be <deploymentProfile>.<environment>`);
    const [deploymentProfile, environment] = segments;
    const { values } = readProfileEnv(profilePath);

    assert.equal(values.SDKWORK_GAMES_PROFILE_ID, profileId, `${profilePath} profile id`);
    assert.equal(values.SDKWORK_GAMES_DEPLOYMENT_PROFILE, deploymentProfile, `${profilePath} deployment profile`);
    assert.equal(values.SDKWORK_GAMES_ENVIRONMENT, environment, `${profilePath} environment`);
    assert.equal(values.VITE_SDKWORK_GAMES_DEPLOYMENT_PROFILE, deploymentProfile, `${profilePath} client profile`);
    assert.ok(!('SDKWORK_GAMES_SERVICE_LAYOUT' in values), `${profilePath} must not expose service layout`);

    assert.ok(!('SDKWORK_GAMES_PLATFORM_API_GATEWAY_AUTOSTART' in values));
  }
});

test('public scripts do not expose process-layout topology flags', () => {
  const packageJson = readJson('package.json');
  const serializedScripts = JSON.stringify(packageJson.scripts);

  assert.ok(!serializedScripts.includes('--service-layout'));
  assert.ok(!serializedScripts.includes('unified-process'));
  assert.ok(!serializedScripts.includes('split-services'));
  assert.ok(!serializedScripts.includes('sdkwork-topology.mjs plan'));
  assert.ok(serializedScripts.includes('sdkwork-topology.mjs print-matrix'));
});

test('production topology env uses structured PostgreSQL config without committed secrets', () => {
  const topology = readJson('specs/topology.spec.json');

  for (const [profileId, profilePath] of Object.entries(topology.profileFiles)) {
    if (!profileId.endsWith('.production')) continue;

    const { source, values } = readProfileEnv(profilePath);

    assert.equal(values.SDKWORK_GAMES_DATABASE_ENGINE, 'postgresql', `${profilePath} database engine`);
    assert.ok(values.SDKWORK_GAMES_DATABASE_HOST, `${profilePath} database host`);
    assert.ok(values.SDKWORK_GAMES_DATABASE_NAME, `${profilePath} database name`);
    assert.ok(values.SDKWORK_GAMES_DATABASE_USERNAME, `${profilePath} database username`);
    assert.ok(values.SDKWORK_GAMES_DATABASE_PASSWORD_FILE, `${profilePath} database password file`);
    assert.ok(values.SDKWORK_GAMES_DATABASE_SSL_MODE, `${profilePath} database ssl mode`);
    assert.ok(!('GAMES_DATABASE_URL' in values), `${profilePath} must not use legacy GAMES_DATABASE_URL`);
    assert.ok(!('SDKWORK_GAMES_DATABASE_PASSWORD' in values), `${profilePath} must not commit database password`);
    assert.ok(!/postgres:\/\/sdkwork:sdkwork@127\.0\.0\.1/u.test(source), `${profilePath} must not commit a local password URL`);
  }
});
