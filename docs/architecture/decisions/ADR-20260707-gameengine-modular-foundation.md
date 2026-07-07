# ADR-20260707-gameengine-modular-foundation

Status: accepted
Requirement: REQ-2026-0001-gameengine-foundation-closeout
Owner: SDKWork maintainers
Date: 2026-07-07
Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, DOMAIN_SPEC.md, API_SPEC.md, SDK_SPEC.md, DATABASE_SPEC.md

## Context

`sdkwork-gameengine` must become the shared foundation for all SDKWork games and game platforms. The
current implementation covers catalog, rooms, and leaderboards, but the target product requires more
foundation capabilities: modes, rules, matchmaking, sessions, points, settlement, events, and
operations.

If these capabilities are added into a single broad service, future games will depend on a low
cohesion module that is difficult to test, scale, or govern. If each concrete game implements its own
rooms, matching, scoring, and ranking, SDKWork will get duplicated contracts and incompatible
operator workflows.

## Decision

Use a modular game foundation architecture:

- Keep `catalog`, `mode`, `rules`, `room`, `matchmaking`, `session`, `points`, `leaderboard`,
  `settlement`, `events`, and `ops` as separate bounded modules.
- Each module owns its domain service, repository port, SQLx repository, API route crate, database
  tables, and tests where applicable.
- Modules communicate through service ports and domain events; repositories do not cross-call other
  repositories.
- Generated app/backend SDKs remain the external consumption boundary.
- Game points and competitive scores belong to game engine; wallet/cash-equivalent balances remain
  external dependency domains.
- Concrete gameplay remains outside the engine and integrates through `sdkwork-gameengine-internal-api`
  first. Public third-party open-api requires a separate product/security approval.
- Leaderboard persistence uses `game_leaderboard_config` for ranking policy and
  `game_leaderboard_entry` for rank projection rows in the pre-GA baseline.

## Alternatives

1. Extend the current three modules only.
   - Benefit: fastest short-term path.
   - Cost: room, leaderboard, session, settlement, and score logic would converge into broad modules.

2. Build a full real-time distributed game runtime immediately.
   - Benefit: powerful long-term runtime.
   - Cost: premature complexity for the current foundation stage; would delay the reusable P0 engine.

3. Let each game implement its own foundation capabilities.
   - Benefit: maximum game-level autonomy.
   - Cost: duplicated APIs, inconsistent scoring, weak operations, and poor platform reuse.

## Consequences

- More module artifacts will be created, but each artifact has a clear owner and test boundary.
- Database design must separate facts, projections, configuration, and audit.
- API design must remain resource-style and SDK-generated across modules.
- Game-server integration starts as internal-api; public open-api is intentionally deferred.
- No compatibility leaderboard table is retained in the pre-GA baseline.

## Verification

- PRD: `docs/product/prd/PRD.md`
- Architecture: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Database design: `docs/architecture/tech/TECH-gameengine-database-design.md`
- Planned implementation verification:
  - `pnpm run db:validate`
  - `pnpm run api:check`
  - `pnpm run sdk:check`
  - `pnpm run verify`
  - `cargo test --workspace`

## Supersedes / Superseded By

None.
