#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const checkOnly = process.argv.includes('--check');

const exportsToGenerate = [
  {
    source: 'apis/app-api/game/games-app-api.openapi.json',
    target: 'generated/openapi/games-app-api.openapi.json',
  },
  {
    source: 'apis/backend-api/game/games-backend-api.openapi.json',
    target: 'generated/openapi/games-backend-api.openapi.json',
  },
];

function readOpenApi(relativePath) {
  const absolutePath = path.join(root, relativePath);
  const document = JSON.parse(fs.readFileSync(absolutePath, 'utf8'));
  if (!document.openapi || !document.paths || !document.components) {
    throw new Error(`${relativePath} is not an OpenAPI document`);
  }
  return document;
}

for (const { source, target } of exportsToGenerate) {
  const document = readOpenApi(source);
  const content = `${JSON.stringify(document, null, 2)}\n`;
  const targetPath = path.join(root, target);
  if (checkOnly) {
    if (!fs.existsSync(targetPath) || fs.readFileSync(targetPath, 'utf8') !== content) {
      process.stderr.write(`[games-openapi] stale generated OpenAPI export: ${target}\n`);
      process.exitCode = 1;
    }
    continue;
  }

  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  fs.writeFileSync(targetPath, content);
}

if (checkOnly) {
  if (process.exitCode) {
    process.stderr.write('[games-openapi] run pnpm run api:materialize to refresh generated OpenAPI exports\n');
  } else {
    process.stdout.write('[games-openapi] generated OpenAPI exports are aligned\n');
  }
} else {
  process.stdout.write('[games-openapi] exported authored OpenAPI documents\n');
}
