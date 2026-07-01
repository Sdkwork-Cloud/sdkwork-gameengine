# SDKWork Games
repository-kind: application

SDKWork games platform application root. Provides game catalog, rooms, and leaderboard capabilities aligned with `../sdkwork-specs`.

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

- **HTTP**: `sdkwork-web-framework` via `crates/sdkwork-gameengine-standalone-gateway`
- **Database**: `sdkwork-database` via `crates/sdkwork-games-database-host` and `database/`
- **Utils**: `@sdkwork/utils` (TypeScript), `sdkwork-utils-rust` (Rust)
- **Discovery**: not integrated (no RPC services yet; add when split-service RPC is required)

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
