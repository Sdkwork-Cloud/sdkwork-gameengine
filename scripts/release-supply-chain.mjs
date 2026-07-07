#!/usr/bin/env node
import { createHash, createSign } from 'node:crypto';
import {
  createReadStream,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { basename, dirname, relative, resolve, sep } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workflow = readJson('sdkwork.workflow.json');

const evidenceDir = resolve(repoRoot, 'dist', 'release-evidence');
const sbomDir = resolve(repoRoot, 'dist', 'sbom');

const command = process.argv[2];

try {
  if (command === 'sign') {
    await signReleaseArtifacts();
  } else if (command === 'sbom') {
    await generateAndSignSbom();
  } else {
    throw new Error('Usage: node scripts/release-supply-chain.mjs <sign|sbom>');
  }
} catch (error) {
  console.error(`[release-supply-chain] ${error.message}`);
  process.exit(1);
}

function readJson(path) {
  return JSON.parse(readFileSync(resolve(repoRoot, path), 'utf8'));
}

function selectedTarget() {
  const targetId = process.env.SDKWORK_PACKAGE_TARGET_ID;
  if (targetId) {
    const target = workflow.targets.find((item) => item.id === targetId);
    if (!target) {
      throw new Error(`Unknown SDKWORK_PACKAGE_TARGET_ID: ${targetId}`);
    }
    return target;
  }
  if (workflow.targets.length === 1) {
    return workflow.targets[0];
  }
  throw new Error('SDKWORK_PACKAGE_TARGET_ID is required when multiple targets exist');
}

function packageId() {
  return process.env.SDKWORK_PACKAGE_ID || selectedTarget().id;
}

function packageVersion() {
  return process.env.SDKWORK_PACKAGE_VERSION || workflow.release?.defaultVersion || '0.0.0';
}

function artifactFilesForTarget(target) {
  const files = new Set();
  for (const pattern of target.outputGlobs ?? []) {
    for (const file of expandGlob(pattern)) {
      if (isReleaseEvidenceFile(file)) {
        continue;
      }
      files.add(file);
    }
  }
  const result = [...files].sort();
  if (result.length === 0) {
    throw new Error(`No release artifacts matched target outputGlobs for ${target.id}`);
  }
  return result;
}

function isReleaseEvidenceFile(file) {
  return (
    file.endsWith('.sha256') ||
    file.endsWith('.sig') ||
    file.endsWith('.cyclonedx.json') ||
    file.endsWith('.spdx.json')
  );
}

function expandGlob(pattern) {
  assertSafeRelativePath(pattern, 'target output glob');
  if (!pattern.includes('*')) {
    const absolute = resolve(repoRoot, pattern);
    return existsSync(absolute) && statSync(absolute).isFile() ? [absolute] : [];
  }

  const root = globSearchRoot(pattern);
  if (!existsSync(root)) {
    return [];
  }
  const regex = globToRegex(normalizePath(resolve(repoRoot, pattern)));
  const matches = [];
  walk(root, (file) => {
    if (regex.test(normalizePath(file)) && statSync(file).isFile()) {
      matches.push(file);
    }
  });
  return matches;
}

function globSearchRoot(pattern) {
  const wildcardIndex = pattern.search(/[*?[]/);
  const prefix = wildcardIndex === -1 ? pattern : pattern.slice(0, wildcardIndex);
  const slashIndex = Math.max(prefix.lastIndexOf('/'), prefix.lastIndexOf('\\'));
  const rootPrefix = slashIndex === -1 ? '.' : prefix.slice(0, slashIndex);
  return resolve(repoRoot, rootPrefix || '.');
}

function globToRegex(pattern) {
  let source = '';
  for (let index = 0; index < pattern.length; index += 1) {
    const char = pattern[index];
    const next = pattern[index + 1];
    if (char === '*' && next === '*') {
      source += '.*';
      index += 1;
    } else if (char === '*') {
      source += '[^/]*';
    } else if (char === '?') {
      source += '[^/]';
    } else {
      source += escapeRegex(char);
    }
  }
  return new RegExp(`^${source}$`);
}

function escapeRegex(value) {
  return value.replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
}

function normalizePath(value) {
  return value.split(sep).join('/');
}

function walk(root, onFile) {
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const fullPath = resolve(root, entry.name);
    if (entry.isDirectory()) {
      walk(fullPath, onFile);
    } else if (entry.isFile()) {
      onFile(fullPath);
    }
  }
}

function assertSafeRelativePath(path, label) {
  if (!path || path.startsWith('/') || /^[A-Za-z]:[\\/]/.test(path)) {
    throw new Error(`${label} must be a safe repository-relative path`);
  }
  const resolved = resolve(repoRoot, path);
  const relativePath = relative(repoRoot, resolved);
  if (relativePath.startsWith('..') || relativePath === '') {
    throw new Error(`${label} must stay inside the repository`);
  }
}

function signingKey() {
  const keyFile = process.env.SDKWORK_RELEASE_SIGNING_PRIVATE_KEY_FILE;
  const keyPem = process.env.SDKWORK_RELEASE_SIGNING_PRIVATE_KEY_PEM;
  if (keyFile) {
    assertSafeRelativePath(keyFile, 'SDKWORK_RELEASE_SIGNING_PRIVATE_KEY_FILE');
    return {
      key: readFileSync(resolve(repoRoot, keyFile), 'utf8'),
      passphrase: process.env.SDKWORK_RELEASE_SIGNING_KEY_PASSPHRASE,
    };
  }
  if (keyPem) {
    return {
      key: keyPem,
      passphrase: process.env.SDKWORK_RELEASE_SIGNING_KEY_PASSPHRASE,
    };
  }
  throw new Error(
    'Signing is required. Provide SDKWORK_RELEASE_SIGNING_PRIVATE_KEY_FILE or SDKWORK_RELEASE_SIGNING_PRIVATE_KEY_PEM from a protected release environment.',
  );
}

async function signReleaseArtifacts() {
  const target = selectedTarget();
  const files = artifactFilesForTarget(target);
  const key = signingKey();
  mkdirSync(evidenceDir, { recursive: true });

  for (const file of files) {
    await writeChecksumAndSignature(file, key);
  }

  console.log(`[release-supply-chain] signed ${files.length} release artifact(s) for ${target.id}`);
}

async function writeChecksumAndSignature(file, key) {
  const hash = createHash('sha256');
  const signer = createSign('sha256');

  await new Promise((resolvePromise, reject) => {
    const stream = createReadStream(file);
    stream.on('data', (chunk) => {
      hash.update(chunk);
      signer.update(chunk);
    });
    stream.on('end', resolvePromise);
    stream.on('error', reject);
  });

  const digest = hash.digest('hex');
  const signature = signer.sign(key, 'base64');
  const checksumPath = `${file}.sha256`;
  const signaturePath = `${file}.sig`;

  writeFileSync(checksumPath, `${digest}  ${basename(file)}\n`, 'utf8');
  writeFileSync(signaturePath, `${signature}\n`, 'utf8');
}

async function generateAndSignSbom() {
  const bom = {
    bomFormat: 'CycloneDX',
    specVersion: '1.6',
    version: 1,
    metadata: {
      timestamp: process.env.SDKWORK_RELEASE_TIMESTAMP || new Date().toISOString(),
      component: {
        type: 'application',
        name: workflow.app?.id || 'sdkwork-games',
        version: packageVersion(),
        purl: `pkg:generic/${workflow.app?.id || 'sdkwork-games'}@${packageVersion()}`,
      },
      properties: [
        { name: 'sdkwork:packageId', value: packageId() },
        { name: 'sdkwork:targetId', value: selectedTarget().id },
      ],
    },
    components: collectComponents(),
  };

  if (bom.components.length === 0) {
    throw new Error('SBOM generation produced no components');
  }

  mkdirSync(sbomDir, { recursive: true });
  const sbomPath = resolve(sbomDir, `${packageId()}.cyclonedx.json`);
  writeFileSync(sbomPath, `${JSON.stringify(bom, null, 2)}\n`, 'utf8');
  await writeChecksumAndSignature(sbomPath, signingKey());

  console.log(`[release-supply-chain] generated SBOM with ${bom.components.length} component(s)`);
}

function collectComponents() {
  const components = new Map();
  for (const component of collectPnpmComponents()) {
    components.set(component.bomRef, component);
  }
  for (const component of collectCargoComponents()) {
    components.set(component.bomRef, component);
  }
  return [...components.values()].sort((left, right) => left.bomRef.localeCompare(right.bomRef));
}

function collectPnpmComponents() {
  const result = spawnJson('pnpm', ['list', '-r', '--json', '--depth', 'Infinity']);
  const components = new Map();
  for (const project of result) {
    visitNpmNode(project.name, project, components);
    for (const [name, dependency] of Object.entries(project.dependencies ?? {})) {
      visitNpmNode(name, dependency, components);
    }
    for (const [name, dependency] of Object.entries(project.devDependencies ?? {})) {
      visitNpmNode(name, dependency, components);
    }
  }
  return [...components.values()];
}

function visitNpmNode(name, node, components) {
  if (!name || !node) {
    return;
  }
  const version = normalizePackageVersion(node.version, node.path);
  const bomRef = `pkg:npm/${encodeURIComponent(name)}@${encodeURIComponent(version)}`;
  if (!components.has(bomRef)) {
    components.set(bomRef, {
      type: 'library',
      name,
      version,
      purl: bomRef,
      bomRef,
      properties: node.path ? [{ name: 'sdkwork:path', value: normalizePath(relative(repoRoot, node.path)) }] : [],
    });
  }
  for (const [dependencyName, dependency] of Object.entries(node.dependencies ?? {})) {
    visitNpmNode(dependencyName, dependency, components);
  }
}

function normalizePackageVersion(version, packagePath) {
  if (version && !version.startsWith('link:') && !version.startsWith('file:')) {
    return version;
  }
  if (packagePath && existsSync(resolve(packagePath, 'package.json'))) {
    const manifest = JSON.parse(readFileSync(resolve(packagePath, 'package.json'), 'utf8'));
    return manifest.version || version || '0.0.0';
  }
  return version || '0.0.0';
}

function collectCargoComponents() {
  const metadata = spawnJson('cargo', ['metadata', '--format-version', '1']);
  return metadata.packages.map((pkg) => {
    const purl = `pkg:cargo/${encodeURIComponent(pkg.name)}@${encodeURIComponent(pkg.version)}`;
    const properties = [];
    if (pkg.source) {
      properties.push({ name: 'sdkwork:cargoSource', value: pkg.source });
    }
    if (pkg.manifest_path) {
      properties.push({
        name: 'sdkwork:manifestPath',
        value: normalizePath(relative(repoRoot, pkg.manifest_path)),
      });
    }
    return {
      type: 'library',
      name: pkg.name,
      version: pkg.version,
      purl,
      bomRef: purl,
      licenses: pkg.license
        ? [
            {
              license: {
                id: pkg.license,
              },
            },
          ]
        : undefined,
      properties,
    };
  });
}

function spawnJson(commandName, args) {
  const result = spawnSync(commandName, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    maxBuffer: 128 * 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(`${commandName} ${args.join(' ')} failed: ${result.stderr || result.stdout}`);
  }
  return JSON.parse(result.stdout);
}
