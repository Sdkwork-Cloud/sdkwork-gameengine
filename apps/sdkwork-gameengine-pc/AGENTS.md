# SDKWork Games PC Guidelines

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing tasks in this PC application root. Follow specs before memory, dictionary before context, stop on ambiguity, and evidence before completion.

## SDKWORK Standards

Canonical SDKWORK specs path from this application root:

- `../../../sdkwork-specs/README.md`
- `../../../sdkwork-specs/SOUL.md`
- `../../../sdkwork-specs/AGENTS_SPEC.md`
- `../../../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../../../sdkwork-specs/NAMING_SPEC.md`
- `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`
- `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`

Do not copy root standard text into this application root. If these relative paths do not resolve, stop and report the broken workspace layout.

## Application Identity

Application manifest: `sdkwork.app.config.json` (`sdkwork-gameengine-pc`, domain `game`, PC browser runtime).

This root is the SDKWork Games PC browser application surface. It composes SDK-backed catalog, rooms, active room listing, global read-only leaderboard, and IAM session runtime. It must not reintroduce unbacked local economy, retired feature packages, local auth, simulated matchmaking, or client-side full-download pagination.

## Local Dictionary Structure

- `AGENTS.md`: PC application agent entrypoint and relative SDKWORK spec index.
- `sdkwork.app.config.json`: PC app manifest, runtime profile, release metadata, and SDK family references.
- `specs/component.spec.json`: PC app root component contract.
- `package.json`: PC app root scripts and package graph.
- `src/`: thin renderer bootstrap and composition boundary.
- `packages/`: PC root packages split by runtime, shell, i18n, commons, and SDK-backed capabilities.
- `README.md`: human index for local development and standards.

## Spec Resolution Order

1. Read this `AGENTS.md`.
2. Read `sdkwork.app.config.json`.
3. Read `specs/component.spec.json`.
4. Read repository root `../../AGENTS.md` and `../../specs/component.spec.json` when the task crosses root contracts.
5. Read `../../../sdkwork-specs/README.md` and only the task-specific root specs.
6. Inspect implementation files only after the dictionary is clear.

Loading is dynamic and progressive: read the nearest dictionary files first, then only the task-specific SDKWORK specs needed for the files being touched. Do not eagerly load every language, runtime, UI, deployment, or SDK spec for unrelated work.

## Required Specs By Task Type

- Agent/workflow: `../../../sdkwork-specs/SOUL.md`, `../../../sdkwork-specs/AGENTS_SPEC.md`, `../../../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.
- Code changes: `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`, plus only the touched language/framework spec.
- TypeScript/Node: `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md`.
- Frontend/React UI: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`, `../../../sdkwork-specs/FRONTEND_SPEC.md`, `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`, `../../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`.
- SDK integration: `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`, `../../../sdkwork-specs/SDK_SPEC.md`, `../../../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`.
- Package scripts and build runners: `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`, `../../../sdkwork-specs/CODE_STYLE_SPEC.md`.
- Release/workflow changes: `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`, `../../../sdkwork-specs/RELEASE_SPEC.md`.

Language-specific specs are on-demand only. Load TypeScript, frontend, Rust, Java, or other language specs only when the task touches those files or their build/runtime contracts.

## Code Style Rules

Follow `../../../sdkwork-specs/CODE_STYLE_SPEC.md` and `../../../sdkwork-specs/NAMING_SPEC.md`. Keep root `src/` thin, keep generated SDK output generator-owned, use package public exports instead of deep imports, and keep PC feature services behind generated SDK/composed facade boundaries.

## Build, Test, and Verification

Run repository-wide verification from `../..` unless a package-local check is explicitly sufficient.

- `pnpm --dir apps/sdkwork-gameengine-pc run typecheck`: PC app TypeScript check.
- `pnpm --dir apps/sdkwork-gameengine-pc run build`: PC renderer build.
- `node ../../../sdkwork-specs/tools/check-frontend-composition.mjs --root ../..`
- `node ../../../sdkwork-specs/tools/check-i18n-standard.mjs --root ../..`
- `node ../../../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace ../..`
- `node ../../tests/contract/gameengine-production-readiness.contract.test.mjs` is not a direct command from this root; run the repository test command from `../..`.

## Agent Execution Rules

Use the local dictionary instead of broad context loading. Do not hand-edit generated SDK output. Do not replace generated SDK calls with raw HTTP. Keep changes scoped to the owning PC package or root bootstrap boundary. Record verification evidence before reporting completion.

## App SDK Consumer Imports

PC application, feature, shell, and service packages must consume HTTP SDKs through composed consumer packages. The application-owned app API client comes from `@sdkwork/gameengine-app-sdk`; IAM clients come from approved `@sdkwork/iam-*` packages. Do not import generated transport package names or generated internals from UI/services.

Before completing SDK integration or frontend service work, run:

```bash
node ../../../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace ../..
```

## HTTP API Response Envelope

All SDKWork-owned HTTP contracts consumed by this PC root follow `../../../sdkwork-specs/API_SPEC.md` sections 4.5 and 14-16. Success is `SdkWorkApiResponse` with numeric `code: 0`, `data`, and `traceId`; errors are `ProblemDetail` with numeric non-zero `code` and `traceId`. UI services must rely on generated SDK behavior and must not construct legacy envelopes or raw HTTP response DTOs.

Before completing API contract, SDK generation, or frontend service work, run from the repository root:

```bash
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
```

## List And Search Pagination

List/search UI must request one server page at a time through generated SDK methods. Do not reintroduce full-download client-side slicing for interactive catalog, rooms, or leaderboard views. Follow `../../../sdkwork-specs/PAGINATION_SPEC.md`.

Before completing list/search UI or SDK service work, run from the repository root:

```bash
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
```

## Human Review Rules

Request human review before breaking SDKWORK standards, changing public naming, altering security/auth behavior, changing generated SDK ownership, or changing production release behavior.
