# SDKWork Game Engine Technical Architecture

Status: draft
Owner: SDKWork maintainers
Updated: 2026-07-08
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md, API_SPEC.md, SDK_SPEC.md, WEB_FRAMEWORK_SPEC.md, WEB_BACKEND_SPEC.md, DATABASE_SPEC.md, DATABASE_FRAMEWORK_SPEC.md, DEPLOYMENT_SPEC.md, APP_SDK_INTEGRATION_SPEC.md

## Document Map

- Product PRD: [../../product/prd/PRD.md](../../product/prd/PRD.md)
- Database design: [TECH-gameengine-database-design.md](TECH-gameengine-database-design.md)
- Root manifest: `sdkwork.app.config.json`
- Runtime topology: `specs/topology.spec.json`
- Current API authorities:
  - `sdkwork-gameengine-app-api`
  - `sdkwork-gameengine-backend-api`
- Current SDK families:
  - `sdkwork-gameengine-app-sdk`
  - `sdkwork-gameengine-backend-sdk`

## 1. Architecture Overview

`sdkwork-gameengine` is the SDKWork game foundation engine. It is an application root with Rust
backend services, Rust route crates, PostgreSQL and SQLite database assets, generated TypeScript SDK
facades, and a PC browser surface. The current production API/UI baseline exposes catalog, rooms,
and read-only leaderboards. Mode, rules, matchmaking, sessions, points, settlement, events, and
audit crates/tables exist as pre-GA foundations and must not be treated as production surfaces until
their route, OpenAPI, SDK, worker, authorization, and UI contracts are complete.

The architecture is module-first:

```text
UI surface
  -> service facade
    -> generated app/backend SDK
      -> route crate
        -> handler
          -> domain service
            -> repository port
              -> sqlx repository / projection store
```

Specific games integrate with the engine through generated SDKs or approved server-facing APIs. They
do not call private repositories, generated internals, or route implementation files.

## 2. Technology Choices

| Layer | Choice | Reason |
| --- | --- | --- |
| Backend runtime | Rust + Axum via `sdkwork-web-framework` | Aligns current route crate and gateway implementation. |
| Database | PostgreSQL and SQLite | Current lifecycle assets, DDL, materialized contract, and validation declare both engines. |
| Database lifecycle | `sdkwork-database` assets under `database/` | Required SDKWork lifecycle, drift, seed, and contract model. |
| API contracts | OpenAPI/route manifests | Source for SDK generation and route verification. |
| SDKs | Generated app/backend TypeScript SDKs with composed facade exports | Keeps consumers away from raw HTTP and generator transport names. |
| PC app | React/Vite under `apps/sdkwork-gameengine-pc` | Current app root and package taxonomy. |
| Runtime topology | standalone unified-process and cloud split-services | Declared in `specs/topology.spec.json`. |

## 3. System Boundaries

### Owned By Game Engine

- Game catalog, modes, rule sets, room lifecycle, room seats.
- Matchmaking tickets, queues, attempts, and results.
- Game sessions, participants, result intake, void/dispute status.
- Game score events, point ledger, point balance projections.
- Leaderboard configuration, leaderboard entries, ranking rebuild state.
- Settlement jobs and reward intents.
- Engine events, webhook delivery state, audit records.
- Backend operations for game foundation workflows.

### Not Owned By Game Engine

- IAM login, tenant, organization, roles, permissions, and session tokens.
- Wallet, recharge, payment, real-money balance, and cash-equivalent ledgers.
- Drive file storage and object storage provider lifecycle.
- Chat, social graph, notification delivery, marketplace, VIP entitlement ledgers.
- Concrete gameplay algorithms, frame sync, rendering, physics, or game server internals.

## 4. Module Architecture

| Module | Rust service crate target | Repository target | Route/API surface |
| --- | --- | --- | --- |
| `catalog` | `sdkwork-game-catalog-service` | `sdkwork-game-catalog-repository-sqlx` | app-api and backend-api |
| `mode` | `sdkwork-game-mode-service` | `sdkwork-game-mode-repository-sqlx` | planned app/backend API expansion |
| `rules` | `sdkwork-game-rules-service` | `sdkwork-game-rules-repository-sqlx` | planned backend API expansion; app read projection where needed |
| `room` | `sdkwork-game-room-service` | `sdkwork-game-room-repository-sqlx` | app-api and backend-api |
| `matchmaking` | `sdkwork-game-matchmaking-service` | `sdkwork-game-matchmaking-repository-sqlx` | planned app/backend API expansion |
| `session` | `sdkwork-game-session-service` | `sdkwork-game-session-repository-sqlx` | planned app/backend/internal API expansion |
| `points` | `sdkwork-game-points-service` | `sdkwork-game-points-repository-sqlx` | planned app/backend API expansion |
| `leaderboard` | `sdkwork-game-leaderboard-service` | `sdkwork-game-leaderboard-repository-sqlx` | app-api currently; backend operations planned |
| `settlement` | `sdkwork-game-settlement-service` | `sdkwork-game-settlement-repository-sqlx` | planned backend API and async workers |
| `events` | `sdkwork-game-events-service` | `sdkwork-game-events-repository-sqlx` | planned backend API and webhook worker |

Current implementation contains `catalog`, `mode`, `rules`, `room`, `matchmaking`, `session`,
`points`, `leaderboard`, `settlement`, and `events` service/repository crates, plus route support,
database host, and standalone gateway crates. Production-mounted routes are limited to health/ready,
catalog, rooms, app leaderboard reads, backend catalog list, and backend room monitoring/forced
close. The `room` module has app-api routes for player room lifecycle and backend-api routes for
operator monitoring and forced close; both are mounted in the standalone gateway.

The `points`, `matchmaking`, `session`, `settlement`, and `events` modules have service/repository
foundations for idempotency, store-level pagination, and projection behavior, but their public route
surfaces, generated SDK methods, workers, and UI facades remain planned. They must not be exposed
through production UI shims or raw HTTP clients.

## 5. Dependency Direction

Allowed dependency direction:

```text
catalog
  -> mode
    -> rules
room -> catalog/mode/rules/player projection
matchmaking -> catalog/mode/rules
session -> room/matchmaking/rules
points -> session/rules
leaderboard -> points/session/season
settlement -> session/points/leaderboard + external reward ports
events -> domain events from all modules
ops -> backend service facade / read models
```

Rules:

- Repositories do not call other repositories.
- Services communicate through explicit service ports or events.
- Read projections may duplicate display fields when the owning source is recorded.
- Cross-domain external dependencies use generated SDKs or approved backend service ports.
- Game engine modules never deep-import generated SDK internals.

## 6. API, SDK, And Data Ownership

### API Authority

| Surface | Authority | Prefix | Audience |
| --- | --- | --- | --- |
| App API | `sdkwork-gameengine-app-api` | `/app/v3/api` | Player-facing clients and game platforms. |
| Backend API | `sdkwork-gameengine-backend-api` | `/backend/v3/api` | Operators and backend-admin tools. |
| Internal API | `sdkwork-gameengine-internal-api` | `/internal/v3/api` | First-party game servers and separately deployed SDKWork game runtimes. |

All SDKWork-owned success responses use the SDKWork v3 envelope. Errors use
`application/problem+json` with numeric codes and trace ids. List/search operations use standard
pagination and store-level filtering. Room and mode player counts are capped at 64 across service
validation, OpenAPI schemas, PostgreSQL DDL, and SQLite DDL so room-seat reads remain bounded.

Currently published app operations:

- `games.health.retrieve`
- `games.ready.retrieve`
- `games.catalog.list`
- `games.catalog.retrieve`
- `games.rooms.list`
- `games.rooms.create`
- `games.rooms.retrieve`
- `games.rooms.seats.list`
- `games.rooms.join`
- `games.rooms.leave`
- `games.rooms.ready`
- `games.rooms.start`
- `games.rooms.close`
- `games.leaderboard.list`
- `games.leaderboard.me.retrieve`

Currently published backend operations:

- `backend.games.catalog.list`
- `backend.games.rooms.list`
- `backend.games.rooms.retrieve`
- `backend.games.rooms.seats.list`
- `backend.games.rooms.forceClose`

### SDK Ownership

| SDK family | Consumer package | Ownership |
| --- | --- | --- |
| `sdkwork-gameengine-app-sdk` | `@sdkwork/gameengine-app-sdk` package facade target | App API client for user-facing clients. |
| `sdkwork-gameengine-backend-sdk` | `@sdkwork/gameengine-backend-sdk` package facade target | Backend API client for backend-admin consumers. |
| future open SDK | Requires explicit approval | Third-party game integration only after product/security review. |

Generated SDK output remains generator-owned. Frontend services consume composed facade exports only.

### PC Production Surface

The PC app mounts only SDK-backed catalog, room creation/lifecycle, and read-only leaderboard
surfaces. IAM runtime owns session state and logout; the local user store is only an IAM session
mirror.

The following surfaces are not part of production until matching API/SDK/service/storage contracts
exist: local auth, local wallet/recharge, VIP/subscription, compute tokens, mall/store, quiz,
claws, ringmatch, arena, tournaments, AI challenge, simulated matchmaking, local recent games,
fake featured banners, local economy ledgers, and client-side pagination over full catalog
downloads.

### Data Ownership

All game engine tables use the `game_` prefix and are owned by `games-platform`. The database
contract is under `database/`. See [TECH-gameengine-database-design.md](TECH-gameengine-database-design.md).

## 7. Runtime Data Flow

### Player Room Flow

```text
PC/H5/mobile UI
  -> game engine service facade
    -> app SDK rooms.create/join/ready/start/close
      -> room app route
        -> room service
          -> room repository
          -> emits game.room.* event
```

### Backend Room Operations Flow

```text
operator console / backend-admin tool
  -> backend SDK rooms.list/retrieve/seats.list/forceClose
    -> room backend route
      -> room service
        -> room repository
          -> closed room state with optimistic concurrency
```

### Match And Session Flow

```text
player creates match ticket
  -> matchmaking queue
    -> match result
      -> session created
        -> game server validates session
          -> game server submits result
            -> session result accepted
              -> settlement job
                -> points ledger
                -> leaderboard projection
                -> reward intent
                -> audit/event/webhook
```

### Score Correction Flow

```text
operator correction command
  -> backend SDK
    -> points service validates authority and reason
      -> append correction ledger entry
        -> rebuild/update leaderboard projection
          -> audit record
```

No flow overwrites historical point ledger entries.

Implemented point ledger behavior:

- Accepted point movement directions are `credit` and `debit`; wallet/cash-equivalent balance
  semantics are rejected at the service boundary.
- `tenant_id + idempotency_key` uniquely identifies an append attempt. Exact replay returns the
  original ledger row and does not update the balance again; conflicting replay returns a conflict.
- Balance projection is tenant-isolated by `ledger_account_id` and records `last_ledger_id`.
- Leaderboard entries are projections and may be upserted or rebuilt from score/point facts without
  mutating ledger history.

## 8. Security, Privacy, And Observability

| Area | Architecture Rule |
| --- | --- |
| Auth | App/backend APIs use SDKWork web framework context and dual-token validation. |
| Server API | Game-server result submission requires declared auth mode, signature/replay protection, and idempotency. |
| Tenant isolation | All tenant data writes use request context tenant/organization, not client-writable tenant fields. |
| Privacy | Store stable IAM ids and optional display-name snapshots only where player UX requires them. |
| Audit | Operator commands, score corrections, settlement retries, voided sessions, and webhook failures produce audit records. |
| Observability | API calls, async jobs, result submissions, and webhook deliveries carry trace ids and structured failure reasons. |
| Rate limiting | Room creation, matchmaking, result submission, point event creation, and backend correction commands require rate-limit tiers. |

## 9. Deployment And Runtime Topology

The root topology supports:

- `standalone.unified-process.development`
- `standalone.unified-process.production`
- `standalone.split-services.development`
- `cloud.split-services.development`
- `cloud.split-services.production`

The application public ingress is currently `sdkwork-gameengine-standalone-gateway`. Cloud profiles
also account for `sdkwork-api-cloud-gateway` as the platform connectivity plane. The cloud
production profile requires the platform API gateway process, and the gameengine cloud gateway
dependency surface uses `apiPrefix = "/app/v3/api"` to match OpenAPI and SDK manifests. Standalone
and cloud must preserve the same API contracts, SDK method shapes, database semantics, and security
behavior.

Release safety is enforced by `sdkwork.workflow.json` and the PC app manifest: checksums,
signatures, SBOMs, and artifact attestations are required. The release lifecycle uses
`scripts/release-supply-chain.mjs` to create checksum/signature evidence and CycloneDX SBOMs; if
the protected signing key is not supplied by CI, the release fails instead of publishing unsigned
artifacts.

## 10. Database Architecture

The database is contract-first:

```text
database/contract/schema.yaml
  -> database/ddl/baseline/postgres/0001_games_baseline.sql
  -> database/ddl/baseline/sqlite/0001_games_baseline.sql
  -> database/migrations/postgres/*
  -> database/migrations/sqlite/*
  -> repositories
  -> API/SDK DTOs
```

Current pre-GA baseline tables:

- `game_catalog`
- `game_mode`
- `game_ruleset`
- `game_room`
- `game_room_seat`
- `game_match_ticket`
- `game_match_result`
- `game_session`
- `game_session_participant`
- `game_session_result`
- `game_score_event`
- `game_point_ledger`
- `game_point_balance`
- `game_leaderboard_config`
- `game_leaderboard_entry`
- `game_settlement_job`
- `game_reward_intent`
- `game_engine_event`
- `game_audit_record`

Target foundation tables are grouped by aggregate:

- Catalog/mode/rules.
- Room/seat.
- Matchmaking ticket/result.
- Session/participant/result.
- Points ledger/projection.
- Leaderboard config/entry.
- Settlement/reward intent.
- Events/webhook/audit.

The current executable baseline includes settlement jobs, reward intents, engine event outbox rows,
and audit records. Webhook subscription/delivery tables and delivery workers remain planned
extensions; route/API/SDK exposure for settlement and events remains part of the API expansion
track.

Leaderboard storage is split in the executable baseline: `game_leaderboard_config` owns ranking
policy and `game_leaderboard_entry` owns rank projection rows. No compatibility leaderboard table
is retained in the pre-GA baseline.

## 11. Directory And Package Layout

| Path | Responsibility |
| --- | --- |
| `apis/` | Authored and materialized API contract inputs. |
| `crates/sdkwork-routes-*-app-api` | App API route/path crates. |
| `crates/sdkwork-routes-*-backend-api` | Backend API route/path crates. |
| `crates/sdkwork-game-*-service` | Domain services and ports. |
| `crates/sdkwork-game-*-repository-sqlx` | SQLx repositories for PostgreSQL/SQLite plus feature-gated memory test stores. |
| `crates/sdkwork-gameengine-standalone-gateway` | Runtime gateway and route composition. |
| `database/` | Database contract, baseline, migrations, seeds, drift policy. |
| `sdks/` | SDK families, route manifests, generated output, composed facades. |
| `apps/sdkwork-gameengine-pc` | PC surface and frontend service composition. |

## 12. Architecture Decision Index

Required decisions before implementation:

| Decision | Status | Notes |
| --- | --- | --- |
| [Modular game engine boundary](../decisions/ADR-20260707-gameengine-modular-foundation.md) | accepted | Keep foundation modules separate from concrete gameplay. |
| Server API posture | accepted | Use `sdkwork-gameengine-internal-api` first; public open-api requires future approval. |
| Points vs wallet boundary | accepted | Game points owned here; wallet/cash-equivalent balances remain external. |
| Leaderboard table split | accepted | Split config and projection in the pre-GA baseline. |
| Realtime adapter timing | accepted | Keep realtime frame sync out of P0/P1 foundation. |

Implementation plan: [../../engineering/plans/PLAN-2026-07-07-gameengine-foundation.md](../../engineering/plans/PLAN-2026-07-07-gameengine-foundation.md)

## 13. Verification

Design verification:

- Product scope reviewed against [../../product/prd/PRD.md](../../product/prd/PRD.md).
- Database design reviewed against [TECH-gameengine-database-design.md](TECH-gameengine-database-design.md).
- API design checked against `API_SPEC.md` before route/OpenAPI changes.
- SDK design checked against `SDK_SPEC.md` and app SDK consumer import rules.

Implementation verification targets:

```bash
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .
pnpm run db:validate
pnpm run api:check
pnpm run sdk:check
pnpm run topology:validate
node --test tests/contract/gameengine-production-readiness.contract.test.mjs
node --test scripts/release-supply-chain.test.mjs
pnpm run check
cargo fmt --all --check
cargo test --workspace
pnpm run verify
```

Database/API/SDK work must additionally run the SDKWork standard validators referenced in
`AGENTS.md` for response envelopes, operation patterns, pagination, and SDK consumer imports.
