import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

const requiredFoundationTables = [
  'game_catalog',
  'game_mode',
  'game_ruleset',
  'game_room',
  'game_room_seat',
  'game_match_ticket',
  'game_match_result',
  'game_session',
  'game_session_participant',
  'game_session_result',
  'game_score_event',
  'game_point_ledger',
  'game_point_balance',
  'game_leaderboard_config',
  'game_leaderboard_entry',
  'game_settlement_job',
  'game_reward_intent',
  'game_engine_event',
  'game_audit_record',
];

function read(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

function createdTables(sql) {
  return Array.from(
    sql.matchAll(/\bCREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-z0-9_]+)/gi),
    (match) => match[1],
  );
}

function registryTables() {
  const registry = JSON.parse(read('database/contract/table-registry.json'));
  return registry.tables.map((table) => table.table_name);
}

test('game engine foundation database contract declares all P0 tables', () => {
  const yaml = read('database/contract/schema.yaml');
  const registry = registryTables();
  const postgresTables = createdTables(
    read('database/ddl/baseline/postgres/0001_games_baseline.sql'),
  );

  for (const tableName of requiredFoundationTables) {
    assert.match(yaml, new RegExp(`name:\\s+${tableName}\\b`), `${tableName} missing from schema`);
    assert.ok(registry.includes(tableName), `${tableName} missing from table registry`);
    assert.ok(postgresTables.includes(tableName), `${tableName} missing from postgres baseline`);
  }
});

test('legacy leaderboard table name and raw cover_url are not part of the pre-GA baseline', () => {
  const postgresSql = read('database/ddl/baseline/postgres/0001_games_baseline.sql');
  const sqliteSql = read('database/ddl/baseline/sqlite/0001_games_baseline.sql');
  const registry = registryTables();

  assert.ok(!createdTables(postgresSql).includes('game_leaderboard'));
  assert.ok(!createdTables(sqliteSql).includes('game_leaderboard'));
  assert.ok(!registry.includes('game_leaderboard'));
  assert.doesNotMatch(postgresSql, /\bcover_url\b/i);
  assert.doesNotMatch(sqliteSql, /\bcover_url\b/i);
});

test('leaderboard repository reads the split leaderboard entry projection', () => {
  const repositorySql = read('crates/sdkwork-game-leaderboard-repository-sqlx/src/sqlx.rs');

  assert.doesNotMatch(repositorySql, /\bFROM\s+game_leaderboard\b/i);
  assert.doesNotMatch(repositorySql, /\bCOUNT\(\*\)\s+FROM\s+game_leaderboard\b/i);
  assert.match(repositorySql, /\bFROM\s+game_leaderboard_entry\b/i);
});

test('bootstrap seed uses foundation tables only', () => {
  const seedSql = read('database/seeds/common/001_bootstrap.sql');

  assert.doesNotMatch(seedSql, /\bINSERT\s+INTO\s+game_leaderboard\b/i);
  assert.match(seedSql, /\bINSERT\s+INTO\s+game_mode\b/i);
  assert.match(seedSql, /\bINSERT\s+INTO\s+game_ruleset\b/i);
  assert.match(seedSql, /\bINSERT\s+INTO\s+game_leaderboard_config\b/i);
  assert.match(seedSql, /\bINSERT\s+INTO\s+game_leaderboard_entry\b/i);
});

test('in-memory repositories are gated to tests and test-support builds', () => {
  const repositoryCrates = [
    'crates/sdkwork-game-catalog-repository-sqlx',
    'crates/sdkwork-game-leaderboard-repository-sqlx',
    'crates/sdkwork-game-mode-repository-sqlx',
    'crates/sdkwork-game-matchmaking-repository-sqlx',
    'crates/sdkwork-game-room-repository-sqlx',
    'crates/sdkwork-game-rules-repository-sqlx',
    'crates/sdkwork-game-session-repository-sqlx',
    'crates/sdkwork-game-settlement-repository-sqlx',
    'crates/sdkwork-game-events-repository-sqlx',
  ];

  for (const crateRoot of repositoryCrates) {
    const cargoToml = read(`${crateRoot}/Cargo.toml`);
    const libRs = read(`${crateRoot}/src/lib.rs`);
    const kindRs = read(`${crateRoot}/src/kind.rs`);

    assert.match(cargoToml, /\[features\][\s\S]*test-support\s*=\s*\[\]/);
    assert.match(libRs, /#\[cfg\(any\(test,\s*feature\s*=\s*"test-support"\)\)\]\s*mod memory;/);
    assert.match(libRs, /#\[cfg\(any\(test,\s*feature\s*=\s*"test-support"\)\)\]\s*pub use memory::/);
    assert.match(kindRs, /#\[cfg\(any\(test,\s*feature\s*=\s*"test-support"\)\)\]\s*use crate::memory::/);
    assert.match(kindRs, /#\[cfg\(any\(test,\s*feature\s*=\s*"test-support"\)\)\]\s*Memory\(/);
  }
});
