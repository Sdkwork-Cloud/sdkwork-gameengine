#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const DEPLOYMENT_PROFILES = new Set(['standalone', 'cloud']);
const RUNTIME_TARGETS = new Set(['browser', 'server', 'container', 'desktop', 'test-runner']);
const DATABASES = new Set(['postgres', 'sqlite']);
const ENVIRONMENTS = new Set(['development', 'test', 'staging', 'production']);

function run(command) {
  const result = spawnSync(command, { cwd: repoRoot, shell: true, stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function parseOptions(rawArgs) {
  const options = {};
  for (let index = 0; index < rawArgs.length; index += 1) {
    const arg = rawArgs[index];
    if (!arg.startsWith('--')) {
      throw new Error(`unsupported positional argument: ${arg}`);
    }

    const [rawKey, inlineValue] = arg.slice(2).split('=', 2);
    const value = inlineValue ?? rawArgs[index + 1];
    if (value == null || value.startsWith('--')) {
      throw new Error(`missing value for --${rawKey}`);
    }
    if (inlineValue == null) {
      index += 1;
    }
    options[rawKey] = value;
  }
  return options;
}

function requireOneOf(options, key, allowed, defaultValue) {
  const value = options[key] ?? defaultValue;
  if (value == null) {
    return undefined;
  }
  if (!allowed.has(value)) {
    throw new Error(`unsupported ${key.replaceAll('-', ' ')}: ${value}`);
  }
  return value;
}

function normalizeAxes(command, options) {
  const deploymentProfile = requireOneOf(
    options,
    'deployment-profile',
    DEPLOYMENT_PROFILES,
    command === 'build' ? 'cloud' : command === 'dev' ? 'standalone' : undefined,
  );
  const runtimeTarget = requireOneOf(
    options,
    'runtime-target',
    RUNTIME_TARGETS,
    command === 'dev' ? 'browser' : command === 'build' ? 'container' : undefined,
  );
  const database = requireOneOf(
    options,
    'database',
    DATABASES,
    command === 'dev' ? 'postgres' : undefined,
  );
  const environment = requireOneOf(
    options,
    'environment',
    ENVIRONMENTS,
    command === 'dev' ? 'development' : command === 'build' ? 'production' : undefined,
  );

  const known = new Set(['deployment-profile', 'runtime-target', 'database', 'environment']);
  for (const key of Object.keys(options)) {
    if (!known.has(key)) {
      throw new Error(`unsupported option: --${key}`);
    }
  }

  return { deploymentProfile, runtimeTarget, database, environment };
}

function logAxes(command, axes) {
  const details = Object.entries(axes)
    .filter(([, value]) => value != null)
    .map(([key, value]) => `${key}=${value}`)
    .join(' ');
  if (details) {
    console.log(`[sdkwork-games] ${command} ${details}`);
  }
}

const args = process.argv.slice(2);
const command = args[0];
let axes;
try {
  axes = normalizeAxes(command, parseOptions(args.slice(1)));
} catch (error) {
  console.error(`[sdkwork-games] ${error.message}`);
  process.exit(1);
}

switch (command) {
  case 'dev':
    logAxes(command, axes);
    run('pnpm --filter sdkwork-gameengine-pc-core typecheck && pnpm --filter sdkwork-gameengine-pc dev');
    break;
  case 'build':
    logAxes(command, axes);
    run('pnpm typecheck && cargo build --workspace --release');
    break;
  case 'test':
    run(
      'pnpm --filter sdkwork-gameengine-pc-core test && node --test scripts/*.test.mjs tests/contract/*.test.mjs && cargo test --workspace',
    );
    break;
  case 'check':
    run(
      'pnpm db:validate && pnpm api:check && pnpm check:pnpm-script-standard && pnpm check:pc-package-naming && pnpm typecheck && pnpm format:rust:check',
    );
    break;
  case 'clean':
    run('cargo clean');
    break;
  default:
    console.error('[sdkwork-games] Usage: node scripts/sdkwork-command.mjs <dev|build|test|check|clean>');
    process.exit(1);
}
