# Production Readiness Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the current pre-launch SDKWork Game Engine root into a verifiable production-readiness baseline without mock production behavior, invalid lifecycle assets, or stale documentation.

**Architecture:** Execute in bounded batches that each improve a concrete SDKWORK gate: database lifecycle, production UI surface, bounded backend behavior, deployment configuration, and documentation canon. Do not add broad new API surfaces until the current published surface is clean and verifiable.

**Tech Stack:** Rust/sqlx, TypeScript/React, pnpm, Cargo, SDKWORK OpenAPI/SDK/database validators.

---

### Task 1: Database Lifecycle Gate

**Files:**
- Modify: `database/seeds/seed.manifest.json`
- Modify: `database/database.manifest.json`
- Modify: `database/contract/schema.yaml`
- Modify: `package.json`
- Test: `tests/contract/database-framework.contract.test.mjs`

- [ ] **Step 1: Run the existing failing verification**

Run: `pnpm run verify`
Expected: FAIL on seed manifest i18n metadata.

- [ ] **Step 2: Fix seed locale metadata and engine declarations**

Add required seed i18n metadata and declare both `postgres` and `sqlite` where lifecycle assets exist.

- [ ] **Step 3: Run database validation**

Run: `pnpm run db:validate`
Expected: PASS.

### Task 2: Production UI Surface Cleanup

**Files:**
- Modify: `apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-shell/src/GamesAppShell.tsx`
- Modify: `apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-commons/src/components/Sidebar.tsx`
- Modify: `tests/contract/gameengine-production-readiness.contract.test.mjs`

- [ ] **Step 1: Add a production readiness contract test**

Assert production shell/sidebar do not expose mock-only, local-ledger, wallet, subscription, compute, mall, AI arena, quiz, ringmatch, or claws views.

- [ ] **Step 2: Verify the test fails**

Run: `node --test tests/contract/gameengine-production-readiness.contract.test.mjs`
Expected: FAIL because those views are currently mounted.

- [ ] **Step 3: Remove production route/sidebar exposure**

Keep only implemented app SDK-backed catalog, rooms, leaderboard, profile/dashboard views. Do not delete package source in this batch; prevent production access first.

- [ ] **Step 4: Verify the test passes**

Run: `node --test tests/contract/gameengine-production-readiness.contract.test.mjs`
Expected: PASS.

### Task 3: Backend Capacity And OOM Safeguards

**Files:**
- Modify: `crates/sdkwork-game-room-service/src/service/mod.rs`
- Modify: `database/ddl/baseline/postgres/0001_games_baseline.sql`
- Modify: `database/ddl/baseline/sqlite/0001_games_baseline.sql`
- Test: Rust room service tests

- [ ] **Step 1: Add a failing max room capacity test**

Run: `cargo test -p sdkwork-game-room-service max_players`
Expected: FAIL for oversized `max_players`.

- [ ] **Step 2: Enforce a documented hard cap**

Reject `max_players` above the production room capacity cap before repository access.

- [ ] **Step 3: Mirror the cap in PostgreSQL and SQLite DDL**

Keep both engines aligned.

- [ ] **Step 4: Run room tests**

Run: `cargo test -p sdkwork-game-room-service`
Expected: PASS.

### Task 4: Production Topology And Release Safety

**Files:**
- Modify: `specs/topology.spec.json`
- Modify: `configs/sdkwork-api-cloud-gateway.gameengine.production.toml`
- Modify: `apps/sdkwork-gameengine-pc/sdkwork.app.config.json`

- [ ] **Step 1: Align cloud production gateway as required**

Set the cloud production ingress process required and align API prefix with OpenAPI.

- [ ] **Step 2: Enable release artifact safety requirements**

Require checksum, signature, and SBOM for PC release metadata.

- [ ] **Step 3: Run topology/package checks**

Run: `pnpm run topology:validate`
Expected: PASS.

### Task 5: Documentation Canon Refresh

**Files:**
- Modify: `docs/product/prd/PRD.md`
- Modify: `docs/architecture/tech/TECH_ARCHITECTURE.md`

- [ ] **Step 1: Remove stale "planned as product surface" language**

Docs must state the production baseline and explicitly separate future expansion from shipped capabilities.

- [ ] **Step 2: Document disabled non-production UI features**

Record that wallet/subscription/compute/mall/gameplay mock modules are not part of production until backed by API/SDK/service contracts.

- [ ] **Step 3: Record production gate requirements**

List the commands that must pass before release.

### Task 6: Full Verification Loop

**Files:**
- No direct edits unless verification exposes gaps.

- [ ] **Step 1: Run narrow gates**

Run database, frontend production readiness, topology, API, SDK consumer, pagination, composition, and Rust room tests.

- [ ] **Step 2: Run full gates**

Run `pnpm run verify`, `cargo fmt --all --check`, and `cargo test --workspace`.

- [ ] **Step 3: Iterate on failures**

Fix only evidenced failures, then rerun the failing gate and relevant aggregate gate.
