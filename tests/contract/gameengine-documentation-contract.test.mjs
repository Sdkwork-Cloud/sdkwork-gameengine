import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function read(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

test('product and architecture docs record resolved foundation decisions', () => {
  const prd = read('docs/product/prd/PRD.md');
  const architecture = read('docs/architecture/tech/TECH_ARCHITECTURE.md');
  const databaseDesign = read('docs/architecture/tech/TECH-gameengine-database-design.md');
  const adr = read('docs/architecture/decisions/ADR-20260707-gameengine-modular-foundation.md');

  for (const [name, content] of Object.entries({ prd, architecture, databaseDesign, adr })) {
    assert.doesNotMatch(content, /\bOpen question\b/i, `${name} still contains open question`);
    assert.doesNotMatch(content, /\bTo be decided\b/i, `${name} still contains undecided API posture`);
  }

  assert.match(prd, /internal-api first/i);
  assert.match(prd, /wallet\/cash-equivalent balances remain external/i);
  assert.doesNotMatch(prd, /\bretrieveMe\b/);
  assert.match(architecture, /sdkwork-gameengine-internal-api/i);
  assert.match(databaseDesign, /pre-GA clean baseline/i);
  assert.match(adr, /Status: accepted/);
});

test('documentation describes split leaderboard tables as the current baseline', () => {
  const docs = [
    read('docs/product/prd/PRD.md'),
    read('docs/architecture/tech/TECH_ARCHITECTURE.md'),
    read('docs/architecture/tech/TECH-gameengine-database-design.md'),
    read('database/README.md'),
  ].join('\n');

  assert.match(docs, /game_leaderboard_config/);
  assert.match(docs, /game_leaderboard_entry/);
  assert.doesNotMatch(docs, /Database:\s*`game_catalog`, `game_room`, `game_leaderboard`/);
  assert.doesNotMatch(docs, /Current active tables:[\s\S]*game_leaderboard`\s*\| Current ranking entry table/);
});
