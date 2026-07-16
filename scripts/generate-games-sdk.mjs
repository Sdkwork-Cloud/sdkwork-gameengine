#!/usr/bin/env node

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const GENERATOR_PACKAGE_NAME = '@sdkwork/sdk-generator';
const GENERATOR_CLI_NAME = 'sdkgen';
const STANDARD_PROFILE = 'sdkwork-v3';
const DEFAULT_WORKSPACE_GENERATOR_ENTRYPOINT = '../sdkwork-sdk-generator/bin/sdkgen.js';
const SUPPORTED_LANGUAGES = new Set(['typescript']);
const EXPECTED_SDK_OWNER = 'sdkwork-gameengine';

const FAMILY_DEFAULTS = {
  'sdkwork-gameengine-app-sdk': { surface: 'app', clientName: 'SdkworkGameengineAppClient' },
  'sdkwork-gameengine-backend-sdk': { surface: 'backend', clientName: 'SdkworkGameengineBackendClient' },
};

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function isBlank(value) {
  return String(value ?? '').trim().length === 0;
}

function parseArgs(argv) {
  const options = {
    dryRun: false,
    languages: [],
    sdkFamily: 'sdkwork-gameengine-app-sdk',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--dry-run') {
      options.dryRun = true;
      continue;
    }
    if (token === '--language') {
      const next = argv[index + 1];
      if (!next || next.startsWith('--')) {
        throw new Error('Missing value for --language.');
      }
      options.languages.push(next);
      index += 1;
      continue;
    }
    if (token === '--sdk-family') {
      const next = argv[index + 1];
      if (!next || next.startsWith('--')) {
        throw new Error('Missing value for --sdk-family.');
      }
      options.sdkFamily = next;
      index += 1;
      continue;
    }
    throw new Error(`Unknown option: ${token}.`);
  }

  return options;
}

function resolveSdkgenEntrypoint(rootDir) {
  const explicit = String(process.env.SDKWORK_SDKGEN_PATH ?? '');
  if (!isBlank(explicit)) {
    return path.resolve(rootDir, explicit);
  }

  const workspaceEntrypoint = path.resolve(rootDir, DEFAULT_WORKSPACE_GENERATOR_ENTRYPOINT);
  if (fs.existsSync(workspaceEntrypoint)) {
    return workspaceEntrypoint;
  }

  throw new Error(
    `Missing ${GENERATOR_PACKAGE_NAME} ${GENERATOR_CLI_NAME} entrypoint. Set SDKWORK_SDKGEN_PATH or check out sdkwork-sdk-generator.`,
  );
}

function createGenerationPlans(rootDir, options) {
  const familyRoot = path.join(rootDir, 'sdks', options.sdkFamily);
  const assemblyPath = path.join(familyRoot, 'sdk-manifest.json');
  const assembly = readJson(assemblyPath);
  assert.equal(assembly.sdkOwner, EXPECTED_SDK_OWNER);

  const requestedLanguages = new Set(
    options.languages.length > 0 ? options.languages : SUPPORTED_LANGUAGES,
  );
  for (const language of requestedLanguages) {
    if (!SUPPORTED_LANGUAGES.has(language)) {
      throw new Error(`Unsupported Games standard SDK language: ${language}.`);
    }
  }

  const input = path.resolve(familyRoot, assembly.generationInputSpec);
  const apiPrefix = assembly.discoverySurface?.apiPrefix ?? '/app/v3/api';
  const defaults = FAMILY_DEFAULTS[options.sdkFamily] ?? FAMILY_DEFAULTS['sdkwork-gameengine-app-sdk'];
  const languages = new Map((assembly.languages ?? []).map((entry) => [entry.language, entry]));

  return [...requestedLanguages].map((language) => {
    const languageEntry = languages.get(language);
    assert.ok(languageEntry, `${options.sdkFamily} must declare ${language} in sdk-manifest.json.`);
    return {
      apiPrefix,
      clientName: languageEntry.consumerSurface?.primaryClient ?? defaults.clientName,
      fixedSdkVersion: String(languageEntry.version ?? assembly.apiVersion ?? '0.1.0'),
      input,
      language,
      output: path.resolve(familyRoot, languageEntry.generatedPath),
      packageName: languageEntry.name,
      sdkName: options.sdkFamily,
      surface: assembly.discoverySurface?.sdkTarget ?? defaults.surface,
    };
  });
}

function runPlan(sdkgenEntrypoint, plan, options) {
  const args = [
    sdkgenEntrypoint,
    'generate',
    '--input',
    plan.input,
    '--output',
    plan.output,
    '--name',
    plan.sdkName,
    '--type',
    plan.surface,
    '--language',
    plan.language,
    '--api-prefix',
    plan.apiPrefix,
    '--sdk-name',
    plan.sdkName,
    '--package-name',
    plan.packageName,
    '--client-name',
    plan.clientName,
    '--fixed-sdk-version',
    plan.fixedSdkVersion,
    '--standard-profile',
    STANDARD_PROFILE,
    ...(options.dryRun ? ['--dry-run'] : []),
  ];

  console.log(`${GENERATOR_CLI_NAME} ${args.slice(1).join(' ')}`);
  const result = spawnSync(process.execPath, args, {
    cwd: process.cwd(),
    stdio: 'inherit',
    windowsHide: true,
  });
  if (result.status !== 0) {
    throw new Error(`${GENERATOR_CLI_NAME} failed for ${plan.sdkName} ${plan.language}.`);
  }
}

export function generateGamesSdk(options = {}, rootDir = process.cwd()) {
  const sdkgenEntrypoint = resolveSdkgenEntrypoint(rootDir);
  const plans = createGenerationPlans(rootDir, options);
  for (const plan of plans) {
    runPlan(sdkgenEntrypoint, plan, options);
  }
  return plans.map((plan) => ({
    language: plan.language,
    output: path.relative(rootDir, plan.output).replace(/\\/g, '/'),
    sdkName: plan.sdkName,
  }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const result = generateGamesSdk(parseArgs(process.argv.slice(2)));
    console.log(JSON.stringify(result, null, 2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
