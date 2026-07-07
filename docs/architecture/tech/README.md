# Technical Architecture Directory

This directory owns the technical architecture Canon for the repository.

## Fixed Entry

- [TECH_ARCHITECTURE.md](TECH_ARCHITECTURE.md) - required entry document. Keep summary, status, and links here.

## Shards

- [TECH-gameengine-database-design.md](TECH-gameengine-database-design.md) - target database design for the game engine foundation.

## Splitting Rules

- Split large architecture content into sibling shards named `TECH-<kebab-topic>.md`.
- Every shard must be linked from `TECH_ARCHITECTURE.md`.
- Do not create competing architecture roots such as `docs/architecture/TECH_ARCHITECTURE.md`; that path is retired and redirect-only.

See `DOCUMENTATION_SPEC.md` section 2.2.
