#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '..');

const API_INPUTS = {
  'app-api': {
    path: 'apis/app-api/game/games-app-api.openapi.json',
    sdkFamily: 'sdkwork-gameengine-app-sdk',
  },
  'backend-api': {
    path: 'apis/backend-api/game/games-backend-api.openapi.json',
    sdkFamily: 'sdkwork-gameengine-backend-sdk',
  },
};

const ROUTE_MANIFESTS = [
  'sdks/_route-manifests/app-api/sdkwork-routes-gameengine-catalog-app-api.route-manifest.json',
  'sdks/_route-manifests/app-api/sdkwork-routes-leaderboard-app-api.route-manifest.json',
  'sdks/_route-manifests/app-api/sdkwork-routes-room-app-api.route-manifest.json',
  'sdks/_route-manifests/backend-api/sdkwork-routes-gameengine-catalog-backend-api.route-manifest.json',
  'sdks/_route-manifests/backend-api/sdkwork-routes-room-backend-api.route-manifest.json',
];

function parseArgs(argv) {
  return { check: argv.includes('--check') };
}

function validateOpenApiContract(surface, inputPath) {
  const fullPath = resolve(repoRoot, inputPath);
  if (!existsSync(fullPath)) {
    console.error(`[games-sdk] missing OpenAPI input for ${surface}: ${inputPath}`);
    return false;
  }

  try {
    const content = JSON.parse(readFileSync(fullPath, 'utf8'));
    if (!content.openapi) {
      console.error(`[games-sdk] ${inputPath} is not a valid OpenAPI document`);
      return false;
    }
    if (!content.components?.securitySchemes?.AuthToken) {
      console.error(`[games-sdk] ${inputPath} is missing dual-token security schemes`);
      return false;
    }
    if (!content.components?.schemas?.ProblemDetail) {
      console.error(`[games-sdk] ${inputPath} is missing ProblemDetail schema`);
      return false;
    }
    console.log(`[games-sdk] OK: ${surface} (${inputPath})`);
    return true;
  } catch (error) {
    console.error(
      `[games-sdk] failed to parse ${inputPath}: ${error instanceof Error ? error.message : String(error)}`,
    );
    return false;
  }
}

function validateSdkFamily(surface, config) {
  const sdkFamilyDir = resolve(repoRoot, 'sdks', config.sdkFamily);
  if (!existsSync(sdkFamilyDir)) {
    console.error(`[games-sdk] missing SDK family directory: sdks/${config.sdkFamily}`);
    return false;
  }
  const assemblyPath = resolve(sdkFamilyDir, 'sdk-manifest.json');
  if (!existsSync(assemblyPath)) {
    console.error(`[games-sdk] missing assembly manifest for ${config.sdkFamily}`);
    return false;
  }
  console.log(`[games-sdk] OK: ${surface} SDK family sdks/${config.sdkFamily}`);
  return true;
}

function validateRouteManifests() {
  let ok = true;
  for (const relativePath of ROUTE_MANIFESTS) {
    const fullPath = resolve(repoRoot, relativePath);
    if (!existsSync(fullPath)) {
      console.error(`[games-sdk] missing route manifest: ${relativePath}`);
      ok = false;
      continue;
    }
    const manifest = JSON.parse(readFileSync(fullPath, 'utf8'));
    for (const route of manifest.routes ?? []) {
      if (route.requestContext !== 'WebRequestContext') {
        console.error(`[games-sdk] ${relativePath} route missing WebRequestContext: ${route.path}`);
        ok = false;
      }
      if (!route.apiSurface) {
        console.error(`[games-sdk] ${relativePath} route missing apiSurface: ${route.path}`);
        ok = false;
      }
    }
  }
  return ok;
}

function runGenerateScript(sdkFamily) {
  const scriptPath = resolve(repoRoot, 'scripts/generate-games-sdk.mjs');
  const result = spawnSync(process.execPath, [scriptPath, '--sdk-family', sdkFamily], {
    cwd: repoRoot,
    stdio: 'inherit',
    windowsHide: true,
  });
  if (result.status !== 0) {
    throw new Error(`Games SDK generation failed for ${sdkFamily}`);
  }
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  let allValid = true;
  for (const [surface, config] of Object.entries(API_INPUTS)) {
    if (!validateOpenApiContract(surface, config.path)) {
      allValid = false;
    }
  }
  if (!validateRouteManifests()) {
    allValid = false;
  }
  if (!allValid) {
    process.exit(1);
  }

  let familiesValid = true;
  for (const [surface, config] of Object.entries(API_INPUTS)) {
    if (!validateSdkFamily(surface, config)) {
      familiesValid = false;
    }
  }

  if (args.check) {
    if (!familiesValid) {
      process.exit(1);
    }
    process.stdout.write('[games-sdk] route manifests, OpenAPI, and SDK families are aligned\n');
    return;
  }

  if (!familiesValid) {
    process.exit(1);
  }

  for (const config of Object.values(API_INPUTS)) {
    runGenerateScript(config.sdkFamily);
  }

  process.stdout.write('[games-sdk] SDK generation pipeline complete\n');
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
