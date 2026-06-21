#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const baselinePath = path.join(root, 'database/ddl/baseline/postgres/0001_games_baseline.sql');
const sql = fs.readFileSync(baselinePath, 'utf8');
const seen = new Set();
const tableNames = [];
for (const match of sql.matchAll(/CREATE TABLE(?: IF NOT EXISTS)? ([a-z0-9_]+)/gi)) {
  const name = match[1];
  if (seen.has(name)) continue;
  seen.add(name);
  tableNames.push(name);
}

const tableRegistry = {
  schemaVersion: 1,
  kind: 'sdkwork.database.table-registry',
  tables: tableNames.map((table_name) => ({
    table_name,
    owner: 'games-platform',
    compliance_level: 'L2',
    lifecycle_status: 'active',
  })),
};

const prefixRegistry = {
  schemaVersion: 1,
  kind: 'sdkwork.database.prefix-registry',
  prefixes: [{ prefix: 'game_', owner: 'games-platform', domain: 'game' }],
};

const schemaYaml = [
  'schema_version: 1',
  'kind: sdkwork.database.schema',
  'module_id: games',
  'contract_version: 1.0.0',
  'owner_team: games-platform',
  'compliance_level: L2',
  'engines:',
  '  - postgres',
  'table_prefix: game_',
  'tables:',
  ...tableNames.map(
    (name) => `  - name: ${name}\n    lifecycle_status: active\n    owner: games-platform`,
  ),
  '',
].join('\n');

fs.writeFileSync(
  path.join(root, 'database/contract/table-registry.json'),
  `${JSON.stringify(tableRegistry, null, 2)}\n`,
);
fs.writeFileSync(
  path.join(root, 'database/contract/prefix-registry.json'),
  `${JSON.stringify(prefixRegistry, null, 2)}\n`,
);
fs.writeFileSync(path.join(root, 'database/contract/schema.yaml'), schemaYaml);

const generatedReadme = path.join(root, 'database/ddl/generated/README.md');
if (!fs.existsSync(generatedReadme)) {
  fs.mkdirSync(path.dirname(generatedReadme), { recursive: true });
  fs.writeFileSync(
    generatedReadme,
    '# Generated DDL\n\nContract-derived DDL output. Do not hand-edit unless your workflow explicitly allows it.\n',
  );
}

process.stdout.write(`materialized ${tableNames.length} tables into games database contract\n`);
