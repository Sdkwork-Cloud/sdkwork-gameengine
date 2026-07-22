#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifests = [
  'sdks/_route-manifests/app-api/sdkwork-routes-gameengine-catalog-app-api.route-manifest.json',
  'sdks/_route-manifests/backend-api/sdkwork-routes-gameengine-catalog-backend-api.route-manifest.json',
  'sdks/_route-manifests/backend-api/sdkwork-routes-room-backend-api.route-manifest.json',
];

for (const relativePath of manifests) {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    console.error(`[games-route-manifest] missing ${relativePath}`);
    process.exit(1);
  }
}

process.stdout.write('[games-route-manifest] route manifests present\n');
