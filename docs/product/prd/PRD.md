# SDKWork Game Engine PRD

Status: draft
Owner: SDKWork maintainers
Application: sdkwork-gameengine
Updated: 2026-07-08
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, DOMAIN_SPEC.md, API_SPEC.md, SDK_SPEC.md

## Document Map

- Technical architecture: [../../architecture/tech/TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md)
- Database design: [../../architecture/tech/TECH-gameengine-database-design.md](../../architecture/tech/TECH-gameengine-database-design.md)
- Current production API/UI baseline: catalog, rooms, read-only leaderboards, IAM-backed session
  identity, PostgreSQL/SQLite database lifecycle, and signed/SBOM-gated release packaging.
- Foundation service baseline: mode, rules, matchmaking, sessions, points, settlement, events, and
  audit crates/tables exist as pre-GA foundations but are not production app UI/API surfaces until
  their OpenAPI, generated SDK, route, worker, and frontend contracts are published.

## 1. Background And Problem

SDKWork needs a reusable game foundation layer that every game, game platform, and future game
developer surface can depend on. Without a shared engine, each game would repeatedly implement game
catalogs, rooms, matchmaking, sessions, scoring, leaderboards, settlement, audit, and operations.
That creates inconsistent rules, duplicated SDK contracts, weak tenant isolation, and expensive
platform integration.

`sdkwork-gameengine` is the shared game foundation module. It owns common game operating
capabilities and exposes them through SDKWork app/backend SDKs. Specific games still own their
gameplay logic; the engine owns the reusable platform capabilities around that gameplay.

Current repository evidence shows a pre-launch production baseline around game catalog, rooms,
read-only leaderboards, IAM session mirroring, bounded room capacity, SQLite/PostgreSQL lifecycle
parity, and release safety:

- Root manifest: `sdkwork.app.config.json` with domain `game` and generated SDK authorities for
  `sdkwork-gameengine-app-api` and `sdkwork-gameengine-backend-api`.
- App API: health/ready, game list/detail, room list/create/retrieve/seats/join/leave/ready/start/
  close, leaderboard list, and my leaderboard entry.
- Backend API: game catalog list plus room list/retrieve/seats/force-close operations for
  operator monitoring and recovery.
- PC production UI: only catalog/room creation and read-only leaderboard surfaces are mounted.
  Retired local-ledger, wallet/recharge, VIP/subscription, compute, mall, quiz, claws, arena,
  ringmatch, tournament, AI challenge, and simulated matchmaking packages are not production
  surfaces.
- Rust service/repository foundations: mode/rules, points, matchmaking, sessions, settlement,
  events, and audit exist for pre-GA expansion. They must not be exposed as product capabilities
  until matching route crates, OpenAPI operations, generated SDK methods, workers, authorization
  policy, and frontend/service facades are complete.
- Database: `game_catalog`, `game_mode`, `game_ruleset`, `game_room`,
  `game_room_seat`, `game_match_ticket`, `game_match_result`, `game_session`,
  `game_session_participant`, `game_session_result`, `game_score_event`,
  `game_point_ledger`, `game_point_balance`, `game_leaderboard_config`,
  `game_leaderboard_entry`, `game_settlement_job`, `game_reward_intent`,
  `game_engine_event`, and `game_audit_record`.

This PRD keeps the production baseline strict while defining the target foundation roadmap.

## 2. Target Users

| User | Need |
| --- | --- |
| Player | Discover games, join rooms, match quickly, play sessions, see scores, ranks, rewards, and history. |
| Game developer | Register a game, configure modes and rules, validate sessions, submit results, and integrate through generated SDKs. |
| Game platform app | Reuse catalog, room, matchmaking, ranking, and player profile flows without rebuilding foundation logic. |
| Operator | Manage games, monitor rooms and queues, inspect sessions, correct scores, rerun settlement, and audit changes. |
| SDKWork platform maintainer | Keep API, SDK, database, security, observability, and deployment boundaries standard across games. |

## 3. Product Positioning

SDKWork Game Engine is a foundation engine, not a concrete game implementation.

It owns:

- Game catalog and mode configuration.
- Room lifecycle and seats.
- Matchmaking queues and tickets.
- Game sessions and result intake.
- Game score and point ledger.
- Leaderboards and ranking read models.
- Rule set versions.
- Settlement orchestration and reward intents.
- Game events, webhook delivery, audit, and operations.

It does not own:

- User login, tenant, organization, roles, or permissions. Those belong to IAM/appbase.
- File binary storage. Cover images, assets, and replays use Drive references.
- Real-money balance, recharge, wallet, or payment ledgers. Those belong to commerce/wallet.
- Specific game rules, physics, battle logic, frame sync, or rendering.
- Chat, notifications, social graph, VIP membership, store merchandise, or entitlement ledgers.

## 4. Goals And Non-Goals

### Goals

1. Provide one high-cohesion, low-coupling foundation engine for all SDKWork games.
2. Let a new game integrate by registering catalog metadata, modes, rule sets, and result submission
   instead of rebuilding rooms, matching, scoring, ranking, and settlement.
3. Keep all public consumption behind generated SDKs and composed facades.
4. Preserve tenant and organization isolation across every resource.
5. Make game scores and rankings auditable, idempotent, and reconstructable.
6. Support standalone and cloud deployment profiles with the same API and SDK contracts.
7. Enable operators to monitor, correct, and recover game operations safely.

### Non-Goals

1. Do not implement gameplay logic for individual games.
2. Do not implement real-time frame sync in the first foundation phase.
3. Do not duplicate IAM login/session behavior.
4. Do not own wallet, payment, recharge, or cash-equivalent balances.
5. Do not store file binaries or object-storage provider fields directly in game tables.
6. Do not provide raw HTTP examples as the primary integration path.
7. Do not expose backend-admin controls to app/client SDKs.

## 5. Scope

### P0 Production Baseline Scope

| Capability | Product Requirement |
| --- | --- |
| Catalog | Players can list and retrieve published game catalog rows through the app SDK. Operators can list catalog rows through the backend SDK. |
| Rooms | Players can list, create, retrieve, inspect seats, join, leave, ready, start, and close rooms through the app SDK. Backend operators can list/retrieve room state, inspect seats, and force close rooms. |
| Capacity | Room and mode player counts are bounded to 1..64 in service validation, OpenAPI schemas, and PostgreSQL/SQLite baseline constraints. |
| Leaderboards | Players can read leaderboard lists and "my rank" through the app SDK. Production UI is read-only until challenge/arena/write APIs exist. |
| Identity | PC UI mirrors IAM session identity and delegates logout to IAM runtime; it does not own local auth, VIP, wallet, or compute balances. |
| Release safety | PC/release metadata requires checksums, signatures, SBOM, and artifact attestations. Missing evidence fails release packaging. |

### P0/P1 Foundation Expansion Scope

These capabilities have database/service foundations or product requirements but are not published
as production UI/API features until route, OpenAPI, SDK, authorization, worker, and frontend
contracts are complete.

| Capability | Product Requirement |
| --- | --- |
| Modes | A game can define one or more modes with player-count limits, rule set binding, matchmaking policy, and leaderboard binding. |
| Points | Game score and point changes are recorded as append-only ledger entries with idempotency and source traceability. |
| Matchmaking | Players or parties can create match tickets, cancel tickets, wait in queues, and receive match results. |
| Sessions | The engine can create a session from a room or match result, track participants, accept results, and drive settlement. |
| Rules | Rule sets version configuration for room, match, score, ranking, and settlement behavior. |
| Settlement | Session results produce score ledger entries, leaderboard updates, and reward intents. |
| Server integration | Game servers can validate sessions and submit signed, idempotent results. |

### P2 Platform Scope

| Capability | Product Requirement |
| --- | --- |
| Seasons | Operators can configure seasons, season windows, ranking resets, and season settlement. |
| Tournaments | Operators can configure registration, brackets, rounds, promotion, and tournament leaderboards. |
| Achievements and missions | Game events can progress achievements and missions and produce reward intents. |
| Moderation | Operators can flag suspicious scores, void sessions, freeze leaderboards, and apply corrective ledger entries. |
| Analytics | Operators can inspect activity, room conversion, match latency, session completion, score distribution, and ranking churn. |

## 6. Capability Model

| Module | Owns | Depends On |
| --- | --- | --- |
| `catalog` | Games, public catalog state, visibility, metadata, Drive references. | IAM context only. |
| `mode` | Game modes, min/max players, rule set binding, match policy binding. | `catalog`, `rules`. |
| `rules` | Rule set versions, typed parameters, rollout status, default policies. | `catalog`. |
| `room` | Room lifecycle, seats, readiness, room visibility, owner/host actions. | `catalog`, `mode`, `rules`, player identity projection. |
| `matchmaking` | Tickets, queues, match attempts, match results, timeout/cancel states. | `catalog`, `mode`, `rules`, optional party projection. |
| `session` | Game sessions, participants, result intake, dispute/void status. | `room`, `matchmaking`, `rules`. |
| `points` | Score events, point ledger, current point projection, correction records. | `session`, `rules`. |
| `leaderboard` | Leaderboard configuration, entry projection, rank query, rebuild status. | `points`, `session`, `season`. |
| `settlement` | Settlement jobs, reward intents, failure/retry state, compensation. | `session`, `points`, `leaderboard`, external wallet/commerce ports. |
| `events` | Engine events, webhook delivery, outbox/inbox status. | All game modules. |
| `ops` | Backend aggregation, audit search, recovery commands. | Generated backend SDK and module services. |

## 7. User Scenarios

### Player Finds And Plays A Game

1. Player opens the game platform.
2. Client calls the app SDK to list available games.
3. Player opens a game detail page and chooses a mode.
4. Player creates or joins a room. Matchmaking starts only after matching ticket APIs and SDK/UI
   surfaces are published.
5. Engine creates a room/match/session boundary.
6. Specific game runtime executes gameplay.
7. Runtime submits the result.
8. Engine updates score ledger, leaderboard projection, reward intents, and audit.
9. Player sees session result, point delta, rank, and reward status.

### Game Developer Integrates A New Game

1. Developer registers a game and configures modes.
2. Developer defines rule set versions and result submission policy.
3. Developer receives SDK/API integration details.
4. Developer validates sessions before gameplay starts.
5. Developer submits signed session results with an idempotency key.
6. Engine processes score, rank, settlement, events, and audit.

### Operator Handles An Abnormal Score

1. Operator searches session and score ledger records by player, game, session, or trace id.
2. Operator sees original result source, applied ledger entries, and leaderboard impact.
3. Operator flags or voids the session if the result is invalid.
4. Engine appends correction ledger entries instead of mutating history.
5. Leaderboard projection rebuilds or adjusts.
6. Audit records the operator, reason, target, trace id, and before/after state.

## 8. API And SDK Product Requirements

### App API

App API is for player-facing apps, PC/H5/mobile clients, and game platform frontends. It uses
`/app/v3/api` and dual-token auth.

Implemented resources:

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

Planned resources:

- `games.modes.list`
- `games.points.me.retrieve`
- `games.points.me.ledger.list`
- `games.matchmaking.tickets.create`
- `games.matchmaking.tickets.cancel`
- `games.matchmaking.tickets.retrieve`
- `games.sessions.retrieve`
- `games.sessions.results.me.retrieve`

Current implementation note: the Rust service/repository foundation for modes, rules, points,
matchmaking tickets, sessions, settlement, events, and audit exists. App-api route crates, OpenAPI
operations, generated SDK methods, workers, and PC service facades for these planned resources
remain in the API/SDK expansion phase.

### Backend API

Backend API is for operator/admin workflows. It uses `/backend/v3/api`, dual-token auth, and
backend-admin permission checks.

Implemented P0 resources:

- `backend.games.catalog.list`
- `backend.games.rooms.list`
- `backend.games.rooms.retrieve`
- `backend.games.rooms.seats.list`
- `backend.games.rooms.forceClose`

Planned P0 operator resources:

- `backend.games.catalog.create`
- `backend.games.catalog.update`
- `backend.games.catalog.publish`
- `backend.games.modes.*`
- `backend.games.points.*` HTTP routes and SDK methods. The service/repository foundation for
  append ledger and retrieve balance is implemented.
- `backend.games.leaderboards.*` HTTP routes and SDK methods. The service/repository foundation for
  config query and entry upsert/rebuild is implemented.
- `backend.games.audit.list`

P1 resources:

- `backend.games.matchmaking.queues.*`
- `backend.games.sessions.*`
- `backend.games.settlements.*`
- `backend.games.rulesets.*`

Current implementation note: `backend.games.matchmaking.*` and `backend.games.sessions.*` have
service/repository foundations, including tenant-scoped SQLx persistence and focused tests. Backend
HTTP route crates and SDK operations are still planned. `backend.games.settlements.*` and
`backend.games.events.*` also have service/repository foundations for settlement jobs, reward
intents, event outbox, and audit search; backend route/API/SDK exposure and worker orchestration are
still planned.

### Server Internal API

Game-server integration is internal-api first for the pre-launch foundation. The approved first
authority is `sdkwork-gameengine-internal-api` under `/internal/v3/api`, for first-party SDKWork
game servers and separately deployed SDKWork runtimes. Public third-party open-api exposure is a
future product/security decision and is not part of the P0/P1 foundation.

Internal resources:

- `game.sessions.validate`
- `game.sessions.results.create`
- `game.points.events.create`
- `game.webhooks.list`
- `game.webhooks.create`

## 9. Database Product Requirements

1. Every foundation table uses the `game_` prefix.
2. Every tenant-owned table stores `tenant_id` and `organization_id`.
3. Append-only facts, including point ledger and audit records, are never physically updated for
   business correction.
4. List/search APIs must be backed by store-level pagination and indexes.
5. Session result, point event, settlement, webhook, and external submission flows require
   idempotency keys.
6. Leaderboards are projections that can be rebuilt from score/point facts.
7. Leaderboards use explicit configuration and projection tables:
   `game_leaderboard_config` defines ranking policy and `game_leaderboard_entry` stores rank
   projection rows.
8. Database design must preserve PostgreSQL and SQLite baseline parity for cloud, standalone, and
   local packaging paths.
9. Current pre-launch baseline must keep PostgreSQL and SQLite DDL, database manifest engines, and
   materialized database contract aligned.
10. Room/mode capacity is capped at 64 players to keep room-seat reads bounded and avoid OOM-prone
    unbounded participant state.

## 10. Success Metrics

| Metric | Target |
| --- | --- |
| New game integration time | A basic game can integrate catalog, room, result submission, points, and leaderboard within one implementation cycle after SDK generation. |
| Duplicate foundation code | New games do not create their own room, ranking, point ledger, or session-result infrastructure. |
| API consistency | App/backend contracts use SDKWork v3 envelopes, standard pagination, and generated SDK resource methods. |
| Audit coverage | Score changes, settlement attempts, operator corrections, and abnormal session handling have traceable audit records. |
| Operational visibility | Operators can inspect active rooms, matching queues, sessions, rankings, and failed settlements. |
| Rebuildability | Leaderboard projections can be rebuilt from point/session facts. |

## 11. Non-Functional Requirements

| Area | Requirement |
| --- | --- |
| Security | Protected app/backend APIs use SDKWork dual-token context. Server-facing APIs use declared open/internal auth and replay protection. |
| Privacy | Store IAM user references only as stable ids and display-name snapshots where needed for ranking UX. |
| Performance | List/search paths use store-level pagination; hot ranking queries use indexed projections. |
| Reliability | Result submission, point events, settlement, and webhook delivery are idempotent and retryable. |
| Observability | Every API and async flow preserves trace id, event id, source id, and failure reason. |
| Portability | Database contract and service boundaries remain compatible with standalone and cloud profiles. |
| Governance | Public naming, generated SDK ownership, security posture, and database migrations require human review before implementation. |

## 12. Production Readiness Gates

Commercial launch is allowed only for the current production baseline: catalog, rooms, read-only
leaderboards, IAM-backed identity, PostgreSQL/SQLite lifecycle parity, cloud/standalone topology,
and signed/SBOM-gated release artifacts.

The following capabilities are excluded from production until their SDKWork-owned API/SDK/service,
authorization, persistence, worker, and UI contracts are complete: wallet/recharge, VIP/
subscription, compute tokens, mall/store, local economy ledgers, quiz, claws, ringmatch, arena,
tournaments, AI challenge, simulated matchmaking, and local auth.

Required release evidence:

- API contracts pass SDKWork response envelope and operation pattern checks.
- List/search paths pass SDKWork pagination checks and do not download all rows for client-side
  slicing.
- App consumers import composed SDK packages only; no raw HTTP or generated transport imports.
- Database manifests, DDL, seeds, and materialized contracts validate for PostgreSQL and SQLite.
- Rust services pass workspace tests and formatting checks.
- PC packages pass typecheck and production-readiness contract tests.
- Topology validates with cloud production gateway required and `/app/v3/api` gateway prefix.
- Release packaging requires checksums, signatures, SBOM, and artifact attestations; missing
  evidence fails the workflow.

Canonical commands:

```bash
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .
pnpm run db:validate
pnpm run api:check
pnpm run topology:validate
node --test tests/contract/gameengine-production-readiness.contract.test.mjs
pnpm run check
cargo fmt --all --check
cargo test --workspace
pnpm run verify
```

## 13. Phases

### Phase 0: Canon And Design

- Fill PRD, architecture, and database design.
- Record resolved product boundaries for server APIs, points ownership, realtime scope, and
  leaderboard baseline.
- Use internal-api first for game-server integration.

### Phase 1: Foundation Closeout

- Add game modes, room commands, point ledger, leaderboard config/entry split, and backend operations.
- Update OpenAPI, SDKs, database contract, Rust services, repositories, and PC service facade.

### Phase 2: Match And Session

- Add matchmaking tickets/queues, sessions, participants, result submission, rulesets, and settlement.
- Current codebase state: matchmaking, session, settlement, events, and audit service/repository
  foundations are implemented. Route/API/SDK exposure, internal server validation/result APIs,
  settlement worker orchestration, and webhook subscription/delivery remain planned.
- Add server validation/result APIs and idempotent processing.

### Phase 3: Platform Operations

- Add seasons, tournaments, achievements, missions, moderation, analytics, and advanced operator tools.

## 14. Linked Requirements

Initial engineering requirement records should be created when implementation starts:

- `REQ-2026-0001-gameengine-foundation-closeout`: catalog/mode/room/points/leaderboard P0.
- `REQ-2026-0002-gameengine-match-session-settlement`: matchmaking/session/result/settlement P1.
- `REQ-2026-0003-gameengine-ops-platform`: operator tooling, season, tournament, moderation P2.

## 15. Resolved Decisions

1. Game-server integration is internal-api first through `sdkwork-gameengine-internal-api`.
   Public third-party open-api is not in scope until an explicit product/security approval exists.
2. Points mean game and competitive points only. Wallet/cash-equivalent balances remain external
   to commerce or wallet domains.
3. Realtime frame sync and authoritative gameplay state remain outside P0/P1 foundation and belong
   to a future realtime adapter.
4. Leaderboard P0 includes game, mode, and optional season dimensions through
   `game_leaderboard_config` and `game_leaderboard_entry`.
5. Public branding is "SDKWork Game Engine"; technical application code remains `games` and SDK
   families remain `sdkwork-gameengine-*`.
6. Player room close remains host-scoped in app-api. Backend forced close is a separate
   backend-admin operation (`backend.games.rooms.forceClose`) that bypasses host ownership but still
   uses tenant context, optimistic concurrency, and a required authenticated operator principal.
