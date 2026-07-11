import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function readJson(path) {
  return JSON.parse(readFileSync(resolve(repoRoot, path), 'utf8'));
}

test('release workflow enforces supply-chain evidence instead of placeholders', () => {
  const workflow = readJson('sdkwork.workflow.json');
  const serialized = JSON.stringify(workflow);

  assert.equal(workflow.security.artifactAttestations, true);
  assert.equal(workflow.security.sbomRequired, true);
  assert.equal(workflow.security.signingRequired, true);
  assert.ok(
    workflow.lifecycle.sign.some((step) => step.run === 'node scripts/release-supply-chain.mjs sign'),
  );
  assert.ok(
    workflow.lifecycle.sbom.some((step) => step.run === 'node scripts/release-supply-chain.mjs sbom'),
  );
  assert.ok(!serialized.includes('placeholder'));
  assert.ok(!serialized.includes('deferred'));
});

test('production topology requires the cloud API gateway', () => {
  const topology = readJson('specs/topology.spec.json');
  const production = topology.orchestration.profiles['cloud.production'];
  const gateway = production.processes.find((process) => process.id === 'platform.api-gateway');

  assert.equal(gateway.required, true);
});

test('gateway app API prefixes match the SDKWork app API root', () => {
  for (const file of [
    'configs/sdkwork-api-cloud-gateway.gameengine.development.toml',
    'configs/sdkwork-api-cloud-gateway.gameengine.production.toml',
  ]) {
    const source = readFileSync(resolve(repoRoot, file), 'utf8');
    assert.ok(source.includes('apiPrefix = "/app/v3/api"'), `${file} must use the app API root`);
    assert.ok(!source.includes('apiPrefix = "/app/v3/api/gameengine"'));
  }
});

test('PC app manifest requires checksum signature and SBOM release evidence', () => {
  const manifest = readJson('apps/sdkwork-gameengine-pc/sdkwork.app.config.json');

  assert.equal(manifest.security.checksumRequired, true);
  assert.equal(manifest.security.signatureRequired, true);
  assert.equal(manifest.security.sbomRequired, true);
});
