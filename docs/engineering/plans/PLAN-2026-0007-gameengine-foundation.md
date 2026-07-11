# Game Engine Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the complete reusable SDKWork Game Engine foundation around catalog, modes, rooms, matchmaking, sessions, points, leaderboards, settlement, events, and operations.

**Architecture:** Keep the engine modular. Each capability owns its service, repository, API route crate, database tables, SDK contract, and focused tests. Generated SDKs and composed service facades are the only supported consumer transport boundary.

**Tech Stack:** Rust, Axum through `sdkwork-web-framework`, SQLx repositories, `sdkwork-database`, OpenAPI/route manifests, generated TypeScript SDKs, React/Vite PC service facade.

---

## File Structure

### Product And Architecture

- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `docs/architecture/tech/TECH-gameengine-database-design.md`
- Modify: `docs/architecture/decisions/ADR-20260707-gameengine-modular-foundation.md`

### Database

- Modify: `database/contract/schema.yaml`
- Modify: `database/contract/table-registry.json`
- Modify: `database/ddl/baseline/postgres/0001_games_baseline.sql`
- Create as needed: `database/migrations/postgres/0001_*` only if production compatibility requires migrations instead of baseline revision.
- Test: `tests/contract/database-framework.contract.test.mjs`

### Rust Services And Repositories

- Existing to extend:
  - `crates/sdkwork-game-catalog-service`
  - `crates/sdkwork-game-catalog-repository-sqlx`
  - `crates/sdkwork-game-room-service`
  - `crates/sdkwork-game-room-repository-sqlx`
  - `crates/sdkwork-game-leaderboard-service`
  - `crates/sdkwork-game-leaderboard-repository-sqlx`
- Create:
  - `crates/sdkwork-game-mode-service`
  - `crates/sdkwork-game-mode-repository-sqlx`
  - `crates/sdkwork-game-rules-service`
  - `crates/sdkwork-game-rules-repository-sqlx`
  - `crates/sdkwork-game-matchmaking-service`
  - `crates/sdkwork-game-matchmaking-repository-sqlx`
  - `crates/sdkwork-game-session-service`
  - `crates/sdkwork-game-session-repository-sqlx`
  - `crates/sdkwork-game-points-service`
  - `crates/sdkwork-game-points-repository-sqlx`
  - `crates/sdkwork-game-settlement-service`
  - `crates/sdkwork-game-settlement-repository-sqlx`
  - `crates/sdkwork-game-events-service`
  - `crates/sdkwork-game-events-repository-sqlx`

### API Routes And SDKs

- Extend:
  - `apis/app-api/game/games-app-api.openapi.json`
  - `apis/backend-api/game/games-backend-api.openapi.json`
  - `sdks/_route-manifests/app-api/*`
  - `sdks/_route-manifests/backend-api/*`
- Create route crates by capability/surface:
  - `crates/sdkwork-routes-mode-app-api`
  - `crates/sdkwork-routes-mode-backend-api`
  - `crates/sdkwork-routes-room-backend-api`
  - `crates/sdkwork-routes-matchmaking-app-api`
  - `crates/sdkwork-routes-matchmaking-backend-api`
  - `crates/sdkwork-routes-session-app-api`
  - `crates/sdkwork-routes-session-backend-api`
  - `crates/sdkwork-routes-points-app-api`
  - `crates/sdkwork-routes-points-backend-api`
  - `crates/sdkwork-routes-leaderboard-backend-api`
  - `crates/sdkwork-routes-settlement-backend-api`
  - `crates/sdkwork-routes-events-backend-api`
- Regenerate:
  - `sdks/sdkwork-gameengine-app-sdk`
  - `sdks/sdkwork-gameengine-backend-sdk`

### PC Surface

- Modify:
  - `apps/sdkwork-gameengine-pc/src/bootstrap/sdkClients.ts`
  - `apps/sdkwork-gameengine-pc/src/bootstrap/gamesProviders.ts`
  - `apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-core`
- Create focused service modules for modes, rooms, matchmaking, sessions, points, leaderboards, and operations when the backend SDK methods exist.

## Task 1: Freeze Product And Architecture Baseline

**Files:**
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Modify: `docs/architecture/tech/TECH-gameengine-database-design.md`

- [x] Record resolved decisions in PRD section 14.
- [x] Adopt internal-api first for game-server integration.
- [x] Confirm `points` excludes all wallet/cash-equivalent balances.
- [x] Adopt pre-GA clean baseline revision for leaderboard split.
- [x] Record approved answers in the PRD and ADR.
- [ ] Commit:

```bash
git add docs/product/prd/PRD.md docs/architecture/tech/TECH_ARCHITECTURE.md docs/architecture/tech/TECH-gameengine-database-design.md docs/architecture/decisions/ADR-20260707-gameengine-modular-foundation.md
git commit -m "docs: define game engine foundation architecture"
```

## Task 2: Materialize Database Contract

**Files:**
- Modify: `database/contract/schema.yaml`
- Modify: `database/contract/table-registry.json`
- Modify: `database/ddl/baseline/postgres/0001_games_baseline.sql`
- Modify: `database/ddl/baseline/sqlite/0001_games_baseline.sql`
- Modify: `database/README.md`

- [x] Add target P0 tables: `game_mode`, `game_ruleset`, `game_room_seat`, `game_score_event`, `game_point_ledger`, `game_point_balance`, `game_leaderboard_config`, `game_leaderboard_entry`, `game_settlement_job`, `game_reward_intent`, `game_engine_event`, `game_audit_record`.
- [x] Replace legacy leaderboard semantic usage with `game_leaderboard_entry` in baseline and repository SQL.
- [x] Use pre-GA clean baseline with no compatibility leaderboard table.
- [x] Add indexes for all list/search and idempotency paths in the P0 foundation tables.
- [x] Run:

```bash
pnpm run db:validate
```

- [x] Expected: command exits 0 with database framework validation passing.
- [ ] Commit:

```bash
git add database/contract database/ddl/baseline/postgres/0001_games_baseline.sql database/README.md
git commit -m "feat: add game engine foundation database contract"
```

## Task 3: Implement Modes And Rules

**Files:**
- Create: `crates/sdkwork-game-mode-service/**`
- Create: `crates/sdkwork-game-mode-repository-sqlx/**`
- Create: `crates/sdkwork-game-rules-service/**`
- Create: `crates/sdkwork-game-rules-repository-sqlx/**`
- Modify: `Cargo.toml`

- [x] Write repository tests for mode list/retrieve/create/update with tenant filtering.
- [x] Write repository tests for active ruleset lookup by game/mode.
- [x] Implement repository ports, production SQLx stores, and feature-gated memory test stores.
- [x] Implement service validation for player counts, status, and ruleset activation.
- [x] Run:

```bash
cargo test -p sdkwork-game-mode-service -p sdkwork-game-mode-repository-sqlx -p sdkwork-game-rules-service -p sdkwork-game-rules-repository-sqlx
```

- [ ] Commit:

```bash
git add Cargo.toml crates/sdkwork-game-mode-service crates/sdkwork-game-mode-repository-sqlx crates/sdkwork-game-rules-service crates/sdkwork-game-rules-repository-sqlx
git commit -m "feat: add game modes and rules services"
```

## Task 4: Complete Room Foundation

**Files:**
- Modify: `crates/sdkwork-game-room-service/**`
- Modify: `crates/sdkwork-game-room-repository-sqlx/**`
- Create: `crates/sdkwork-routes-room-backend-api/**`
- Modify: `crates/sdkwork-routes-room-app-api/**`

- [x] Add room create/join/leave/ready/start/close service commands.
- [x] Add `game_room_seat` repository operations.
- [x] Enforce capacity, status transitions, host permissions, and optimistic concurrency.
- [x] Add app-api route operations for player room commands.
- [x] Add backend-api route operations for room monitoring and forced close.
- [x] Run:

```bash
cargo test -p sdkwork-game-room-service -p sdkwork-game-room-repository-sqlx -p sdkwork-routes-room-app-api -p sdkwork-routes-room-backend-api
```

- [ ] Commit:

```bash
git add crates/sdkwork-game-room-service crates/sdkwork-game-room-repository-sqlx crates/sdkwork-routes-room-app-api crates/sdkwork-routes-room-backend-api
git commit -m "feat: complete game room lifecycle"
```

## Task 5: Add Points And Leaderboard Projections

**Files:**
- Create: `crates/sdkwork-game-points-service/**`
- Create: `crates/sdkwork-game-points-repository-sqlx/**`
- Modify: `crates/sdkwork-game-leaderboard-service/**`
- Modify: `crates/sdkwork-game-leaderboard-repository-sqlx/**`
- Modify: `crates/sdkwork-routes-leaderboard-app-api/**`

- [x] Write ledger append tests with idempotency.
- [x] Write point balance projection tests.
- [x] Write leaderboard entry update/rebuild tests.
- [x] Implement append-only point ledger operations.
- [x] Implement leaderboard config and entry query operations.
- [x] Run:

```bash
cargo test -p sdkwork-game-points-service -p sdkwork-game-points-repository-sqlx -p sdkwork-game-leaderboard-service -p sdkwork-game-leaderboard-repository-sqlx -p sdkwork-routes-leaderboard-app-api
```

- [x] Expected: command exits 0 with points, leaderboard, and leaderboard route crates passing.

- [ ] Commit:

```bash
git add crates/sdkwork-game-points-service crates/sdkwork-game-points-repository-sqlx crates/sdkwork-game-leaderboard-service crates/sdkwork-game-leaderboard-repository-sqlx
git commit -m "feat: add game points and leaderboard projections"
```

## Task 6: Add Matchmaking And Sessions

**Files:**
- Create: `crates/sdkwork-game-matchmaking-service/**`
- Create: `crates/sdkwork-game-matchmaking-repository-sqlx/**`
- Create: `crates/sdkwork-game-session-service/**`
- Create: `crates/sdkwork-game-session-repository-sqlx/**`

- [x] Add match ticket create/cancel/retrieve tests.
- [x] Add queue selection tests with store-level pagination/index-aware queries.
- [x] Add session create/start/result-submit tests.
- [x] Add idempotency tests for submitted session results.
- [x] Implement matchmaking and session services.
- [x] Run:

```bash
cargo test -p sdkwork-game-matchmaking-service -p sdkwork-game-matchmaking-repository-sqlx -p sdkwork-game-session-service -p sdkwork-game-session-repository-sqlx
```

- [x] Expected: command exits 0 with matchmaking and session service/repository crates passing.

- [ ] Commit:

```bash
git add crates/sdkwork-game-matchmaking-service crates/sdkwork-game-matchmaking-repository-sqlx crates/sdkwork-game-session-service crates/sdkwork-game-session-repository-sqlx
git commit -m "feat: add game matchmaking and sessions"
```

## Task 7: Add Settlement, Events, And Audit

**Files:**
- Create: `crates/sdkwork-game-settlement-service/**`
- Create: `crates/sdkwork-game-settlement-repository-sqlx/**`
- Create: `crates/sdkwork-game-events-service/**`
- Create: `crates/sdkwork-game-events-repository-sqlx/**`

- [x] Add settlement job retry tests.
- [x] Add reward intent idempotency tests.
- [x] Add event outbox tests.
- [x] Add audit append/search tests.
- [x] Implement settlement orchestration ports without direct wallet/commerce table writes.
- [x] Run:

```bash
cargo test -p sdkwork-game-settlement-service -p sdkwork-game-settlement-repository-sqlx -p sdkwork-game-events-service -p sdkwork-game-events-repository-sqlx
```

- [x] Expected: command exits 0 with settlement/events service and repository crates passing.

- [ ] Commit:

```bash
git add crates/sdkwork-game-settlement-service crates/sdkwork-game-settlement-repository-sqlx crates/sdkwork-game-events-service crates/sdkwork-game-events-repository-sqlx
git commit -m "feat: add game settlement and engine events"
```

## Task 8: Expand API Contracts And SDKs

**Files:**
- Modify: `apis/app-api/game/games-app-api.openapi.json`
- Modify: `apis/backend-api/game/games-backend-api.openapi.json`
- Modify: `sdks/_route-manifests/**`
- Modify: `sdks/sdkwork-gameengine-app-sdk/**`
- Modify: `sdks/sdkwork-gameengine-backend-sdk/**`

- [ ] Add OpenAPI operations with SDKWork v3 response envelopes.
- [ ] Add standard list/search pagination for collection routes.
- [ ] Add route manifests with `WebRequestContext`, surface, owner, authority, and auth mode.
- [ ] Run:

```bash
pnpm run api:check
pnpm run sdk:generate
pnpm run sdk:check
```

- [ ] Commit:

```bash
git add apis sdks
git commit -m "feat: expand game engine API and SDK surfaces"
```

## Task 9: Wire Gateway And PC Services

**Files:**
- Modify: `crates/sdkwork-gameengine-standalone-gateway/src/bootstrap/**`
- Modify: `crates/sdkwork-gameengine-standalone-gateway/src/lib.rs`
- Modify: `apps/sdkwork-gameengine-pc/src/bootstrap/**`
- Modify: `apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-core/**`

- [ ] Mount new route crates in standalone gateway.
- [ ] Inject generated app/backend SDK clients through bootstrap.
- [ ] Add PC core service facades for modes, rooms, matchmaking, sessions, points, and leaderboards.
- [ ] Run:

```bash
pnpm run check
cargo test -p sdkwork-gameengine-standalone-gateway
```

- [ ] Commit:

```bash
git add crates/sdkwork-gameengine-standalone-gateway apps/sdkwork-gameengine-pc
git commit -m "feat: wire game engine gateway and pc services"
```

## Task 10: Final Verification

**Files:**
- All touched files.

- [ ] Run:

```bash
pnpm run db:validate
pnpm run api:check
pnpm run sdk:check
pnpm run check
pnpm run verify
cargo fmt --all --check
cargo test --workspace
```

- [ ] Run SDKWork validators required by `AGENTS.md` for touched API/SDK/list work.
- [ ] Update release/changelog or migration evidence if the database/API contract changed after release.
- [ ] Commit final evidence updates.
