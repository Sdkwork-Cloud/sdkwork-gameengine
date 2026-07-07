# Games Database

`sdkwork-database` lifecycle assets for module `games` (`game_` table prefix).

## Design

- Product PRD: [../docs/product/prd/PRD.md](../docs/product/prd/PRD.md)
- Technical architecture: [../docs/architecture/tech/TECH_ARCHITECTURE.md](../docs/architecture/tech/TECH_ARCHITECTURE.md)
- Target database design: [../docs/architecture/tech/TECH-gameengine-database-design.md](../docs/architecture/tech/TECH-gameengine-database-design.md)

## Owner

- Team: games-platform
- Module id: `games`
- Service code: `GAMES`

## Engines

- Primary: PostgreSQL
- Dev/test: SQLite parity is a design target when local/standalone packaging requires it.

## Initialization State

This module is in initialization state for greenfield deployments:

1. Baseline: `database/ddl/baseline/{engine}/0001_games_baseline.sql` contains the full DDL snapshot.
2. Migrations: `database/migrations/{engine}/` is reserved for post-GA incremental schema changes.
3. Drift: run `pnpm db:drift:check` before release.

The executable baseline is the pre-GA clean baseline. Leaderboards are already split into
`game_leaderboard_config` for ranking policy and `game_leaderboard_entry` for rank projection rows;
there is no compatibility leaderboard table in this application state.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
