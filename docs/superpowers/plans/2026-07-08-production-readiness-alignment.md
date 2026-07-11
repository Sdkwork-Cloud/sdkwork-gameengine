# Production Readiness Alignment Record

Status: completed
Date: 2026-07-08
Application: sdkwork-gameengine

## Objective

Bring the pre-launch SDKWork Game Engine root into a verifiable production-readiness baseline:
SDKWork-standard API envelopes, SDK-backed PC surfaces, bounded list behavior, PostgreSQL/SQLite
lifecycle parity, hardened production topology, and documentation aligned with the actual shipped
scope.

## Implemented Scope

- Database lifecycle metadata now declares active locale ownership, PostgreSQL and SQLite lifecycle
  assets, and database framework validation through `pnpm run db:validate`.
- Published API contracts stay behind generated SDK families and pass SDKWork response envelope,
  operation pattern, pagination, route manifest, and SDK generation checks.
- Room capacity is bounded in service validation, OpenAPI schemas, PostgreSQL DDL, and SQLite DDL.
- Production PC navigation exposes only SDK-backed catalog, room, active-room, IAM session mirror,
  and global read-only leaderboard surfaces.
- Retired local-only PC packages, route entries, unsupported leaderboard segments, and inactive
  i18n dictionaries are removed from the production package graph.
- Runtime topology uses SDKWork v4 two-segment profiles: `standalone.development`,
  `standalone.production`, `cloud.development`, and `cloud.production`.
- Production cloud gateway config uses explicit public hosts, no loopback upstream, restricted CORS,
  upstream readiness checks, metrics, tracing, WAF, rate limiting, and circuit breaker protection.
- Database runtime config uses structured `SDKWORK_GAMES_DATABASE_*` keys. Production requires an
  explicit database config and a password file for structured PostgreSQL config.
- List/search service boundaries now reject invalid SDKWork pagination input before repository
  access. Requests with `page_size > 200`, `page = 0`, or forbidden pre-launch query aliases are
  rejected instead of being silently clamped or defaulted.
- App/backend list routes use a shared strict query extractor for list/search endpoints. Forbidden
  HTTP query aliases such as `pageSize`, `limit`, `page_no`, `pageNo`, `per_page`, and `size` now
  return SDKWork `application/problem+json` with numeric `40003 INVALID_PARAMETER` and a propagated
  `traceId`.
- The root build dispatcher rejects retired topology axis values before running workflows and runs
  production builds through `cargo build --workspace --release`.
- The PC app root pins React ambient types to its own dependency graph and deduplicates `react` and
  `react-dom` at Vite runtime resolution, preventing linked IAM/UI packages from introducing a
  second React type/runtime instance.
- Repository and PC app agent dictionaries now pass SDKWORK agent/workflow validation, and the PC app
  root owns its local `specs/component.spec.json` contract.
- Product and technical architecture docs now describe the implemented baseline and explicitly
  exclude unsupported roadmap capabilities until their API, SDK, storage, worker, authorization,
  and UI contracts are complete.

## Verification Evidence

Fresh verification was run from `E:\sdkwork-space\sdkwork-gameengine`:

```bash
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .
pnpm run db:validate
pnpm run topology:validate
pnpm run gateway:validate:cloud
node --test tests/contract/gameengine-production-readiness.contract.test.mjs
node --test scripts/topology-profile-contract.test.mjs
node --test scripts/release-supply-chain.test.mjs
node --test scripts/games-workspace-boundary.test.mjs
pnpm --dir apps/sdkwork-gameengine-pc run typecheck
cargo fmt --all --check
cargo test -p sdkwork-games-database-host
cargo clippy -p sdkwork-games-database-host --all-targets -- -D warnings
cargo test -p sdkwork-game-catalog-service list_games_rejects_invalid_pagination_before_repository_access
cargo test -p sdkwork-game-room-service -p sdkwork-game-leaderboard-service -p sdkwork-game-mode-service -p sdkwork-game-matchmaking-service -p sdkwork-game-settlement-service -p sdkwork-game-events-service invalid_pagination_before_repository_access
cargo test -p sdkwork-routes-catalog-app-api catalog_list_invalid_page_size_returns_invalid_parameter_problem
cargo test -p sdkwork-routes-catalog-app-api -p sdkwork-routes-room-app-api -p sdkwork-routes-room-backend-api -p sdkwork-routes-leaderboard-app-api forbidden_pagination_aliases
cargo test -p sdkwork-gameengine-standalone-gateway catalog_router_rejects_forbidden_pagination_alias_with_problem_json
cargo test -p sdkwork-routes-games-support -p sdkwork-routes-catalog-app-api -p sdkwork-routes-catalog-backend-api -p sdkwork-routes-room-app-api -p sdkwork-routes-room-backend-api -p sdkwork-routes-leaderboard-app-api -p sdkwork-gameengine-standalone-gateway
node ../sdkwork-specs/tools/check-agent-workflow-standard.mjs --root .
node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root .
node ../sdkwork-specs/tools/check-frontend-composition.mjs --root .
node ../sdkwork-specs/tools/check-permission-composition.mjs --root .
node ../sdkwork-specs/tools/check-application-layering.mjs --root .
node ../sdkwork-specs/tools/check-repository-docs-standard.mjs --root .
node ../sdkwork-specs/tools/check-identity-naming.mjs --root .
node ../sdkwork-specs/tools/check-workspace-lock-package-paths.mjs --root .
node ../sdkwork-specs/tools/check-workspace-member-protocol.mjs --root .
node ../sdkwork-specs/tools/check-app-runtime-hosting-debt.mjs --workspace ..
node ../sdkwork-specs/tools/check-topology-deployment-profiles.mjs --workspace .. --repo sdkwork-gameengine
pnpm test
pnpm run check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm run verify
pnpm run build
```

The aggregate production-readiness contract covers the PC shell, sidebar, topbar, game center,
active-room grid, leaderboard, i18n bundle, removed package directories, IAM session mirror, and
IAM-owned logout flow.

The shared topology implementation was also revalidated from `E:\sdkwork-space\sdkwork-app-topology`
with `pnpm test` and `pnpm run validate:example` because the gameengine topology profiles consume
that shared resolver.

## Commercialization Boundary

Commercial launch readiness applies only to the documented P0 baseline: catalog, rooms, active-room
display, global read-only leaderboard, IAM-backed identity, PostgreSQL/SQLite lifecycle, SDKWork v4
topology, hardened gateway config, and signed/SBOM-gated release packaging.

Roadmap capabilities remain excluded from launch until they are complete end to end through
requirements, API, OpenAPI, generated SDK, backend service/repository implementation, worker or
async orchestration where needed, authorization policy, frontend service/UI, documentation, and
verification gates.
