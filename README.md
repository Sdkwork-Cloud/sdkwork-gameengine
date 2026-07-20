# SDKWork Games
repository-kind: application

SDKWork Game Engine application root. Provides reusable game catalog, modes, rules, rooms, points,
leaderboard, audit, and operations foundations aligned with `../sdkwork-specs`.

## Active layout

| Path | Purpose |
| --- | --- |
| `apis/` | HTTP API contracts (open/app/backend) |
| `apps/sdkwork-gameengine-pc/` | PC browser/desktop React application root |
| `crates/` | Rust... Rust services, repositories, API server |
| `database/` | `sdkwork-database` lifecycle assets (`moduleId=games`, prefix `game_`) |
| `sdks/` | SDK families and route manifests |
| `scripts/`, `tools/` | Verification, generation, and command dispatch |
| `deployments/` | Deployment descriptors and packaging handoff |
| `configs/` | Safe runtime config templates |

## Framework integration

- **HTTP**: `sdkwork-web-framework` via `crates/sdkwork-api-gameengine-standalone-gateway` (catalog, room, leaderboard)
- **Database**: `sdkwork-database` via `crates/sdkwork-games-database-host` and `database/`
  (`game_catalog`, `game_mode`, `game_ruleset`, `game_room`, `game_room_seat`,
  `game_score_event`, `game_point_ledger`, `game_point_balance`,
  `game_leaderboard_config`, `game_leaderboard_entry`, `game_audit_record`)
- **Utils**: `@sdkwork/utils` (TypeScript), `sdkwork-utils-rust` (Rust)
- **PC services**: `sdkwork-gameengine-pc-core` exposes `catalogService`, `roomService`, `leaderboardService` for consumer apps
- **Discovery**: not integrated (no RPC services yet; add when split-service RPC is required)

Consumer applications (for example `sdkwork-games`) depend on this repository for platform APIs and re-export PC services from `sdkwork-gameengine-pc-core`.

## Commands

```bash
pnpm install
pnpm dev
pnpm verify
pnpm api:materialize
pnpm db:validate
```

See `AGENTS.md` and `../sdkwork-specs/README.md` for standards.

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
