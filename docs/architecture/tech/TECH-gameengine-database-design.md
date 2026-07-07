# SDKWork Game Engine Database Design

Status: draft
Owner: SDKWork maintainers
Updated: 2026-07-08
Specs: DATABASE_SPEC.md, DATABASE_FRAMEWORK_SPEC.md, API_SPEC.md, PAGINATION_SPEC.md, SECURITY_SPEC.md

## 1. Purpose

This document defines the target database design for the SDKWork Game Engine foundation. It is a
review document for schema evolution. It does not by itself change the executable baseline DDL.

Executable database assets remain under `database/`:

- `database/database.manifest.json`
- `database/contract/schema.yaml`
- `database/ddl/baseline/postgres/0001_games_baseline.sql`
- `database/ddl/baseline/sqlite/0001_games_baseline.sql`
- `database/migrations/postgres/`
- `database/migrations/sqlite/`
- `database/drift/policy.yaml`

## 2. Current Baseline

Current active pre-GA clean baseline tables:

| Table | Current role | Notes |
| --- | --- | --- |
| `game_catalog` | Game master record and catalog metadata. | Includes Drive/media reference payload and default mode reference. |
| `game_mode` | Playable mode under a game. | Owns player-count limits, room/match flags, and active ruleset binding. |
| `game_ruleset` | Versioned rule configuration. | Stores scoring, room, match, ranking, and settlement parameters. |
| `game_room` | Room aggregate root. | Owns lifecycle, host, visibility, mode, ruleset, and occupancy projection. |
| `game_room_seat` | Room participant state. | Owns seat, readiness, and player snapshot inside a room. |
| `game_match_ticket` | Matchmaking request. | Owns queued/cancelled ticket state, idempotent creation, player lookup, and priority queue ordering. |
| `game_match_result` | Matchmaking output. | Durable output that can later create sessions; table exists in the pre-GA baseline. |
| `game_session` | Gameplay session lifecycle. | Owns session identity, room/match source references, status, server id, metadata, and result version. |
| `game_session_participant` | Session participant snapshot. | Owns player/team snapshot and per-session result projection. |
| `game_session_result` | Submitted result intake. | Owns source, idempotency key, payload hash, validation state, and raw result payload. |
| `game_score_event` | Validated score fact. | Append-only score source for points and rankings. |
| `game_point_ledger` | Game point ledger. | Append-only game/competitive point movement; not wallet balance. |
| `game_point_balance` | Current point projection. | Rebuildable projection by player/game/mode/season. |
| `game_leaderboard_config` | Leaderboard definition. | Owns ranking metric, ordering, reset policy, and scope. |
| `game_leaderboard_entry` | Leaderboard rank projection. | Stores per-player rank rows, sourced from point/score facts. |
| `game_settlement_job` | Settlement orchestration state. | Owns session/result settlement lifecycle, idempotency, retry state, and failure detail. |
| `game_reward_intent` | External reward grant intent. | Stores reward grant intents only; downstream wallet/commerce/inventory/entitlement tables remain externally owned. |
| `game_engine_event` | Engine event outbox. | Stores idempotent game engine events for async publishing and retry. |
| `game_audit_record` | Operator/system audit trail. | Records corrections, moderation, and recovery commands. |

Current contract version: `1.0.0`.

## 3. Design Principles

1. Use `game_` table prefix for every engine-owned table.
2. Use `tenant_id` on every tenant-owned row.
3. Use `organization_id` where organization-scoped filtering or ownership can apply.
4. Keep facts append-only when they affect points, settlement, audit, or external result intake.
5. Store current-state projections separately from historical facts.
6. Use explicit status fields and status timestamps for state machines.
7. Use idempotency keys for result, point, settlement, webhook, and server-origin commands.
8. Store-level pagination must be supported by indexes.
9. Do not store Drive/provider raw file fields; store Drive/media references as JSON or stable ids.
10. Do not store wallet/payment balance facts in game tables.
11. Keep PostgreSQL and SQLite baseline DDL and materialized contract engines aligned.
12. Bound room/mode player capacity to 1..64 at service, API schema, and database constraint layers.

## 4. Target Table Catalog

### Catalog And Rules

| Table | Profile | Purpose |
| --- | --- | --- |
| `game_catalog` | tenant_entity | Game master record and catalog metadata. |
| `game_mode` | tenant_entity | A playable mode under a game. |
| `game_ruleset` | dictionary_entity | Versioned rule configuration for modes, rooms, matching, scoring, and settlement. |
| `game_player` | projection | Game-domain player projection referencing IAM user ids. |

### Room And Matchmaking

| Table | Profile | Purpose |
| --- | --- | --- |
| `game_room` | tenant_entity | Room aggregate root. |
| `game_room_seat` | relation_entity | Seat/participant state inside a room. |
| `game_match_ticket` | event_log | A player's or party's active/past matchmaking request. |
| `game_match_result` | event_log | Matchmaking output that can create a session. |

### Sessions And Scores

| Table | Profile | Purpose |
| --- | --- | --- |
| `game_session` | tenant_entity | One gameplay session/match instance. |
| `game_session_participant` | relation_entity | Participant snapshot and final result in a session. |
| `game_session_result` | event_log | Raw submitted session result with idempotency and signature metadata. |
| `game_score_event` | event_log | Validated score event derived from session/manual/system sources. |
| `game_point_ledger` | ledger_entry | Append-only point ledger. |
| `game_point_balance` | projection | Current point balance/rating projection by player/game/mode/season. |

### Leaderboards And Seasons

| Table | Profile | Purpose |
| --- | --- | --- |
| `game_leaderboard_config` | dictionary_entity | Leaderboard definition/configuration. |
| `game_leaderboard_entry` | projection | Rank projection by leaderboard/player. |
| `game_leaderboard_rebuild_job` | event_log | Planned rebuild or recalculation job state; not in the current pre-GA baseline. |
| `game_season` | tenant_entity | Planned season definition and time window; not in the current pre-GA baseline. |

### Settlement, Events, And Operations

| Table | Profile | Purpose |
| --- | --- | --- |
| `game_settlement_job` | event_log | Settlement orchestration state for a session/result. |
| `game_reward_intent` | event_log | Intent to grant external rewards through wallet/commerce/inventory dependencies. |
| `game_engine_event` | outbox_event | Domain event outbox for engine events. |
| `game_webhook_subscription` | tenant_entity | Game-server or operator webhook subscription. |
| `game_webhook_delivery` | event_log | Webhook delivery attempts and status. |
| `game_audit_record` | audit_log | Operator and system audit trail. |

## 5. Table Designs

### 5.1 `game_catalog`

Purpose: game master data and public catalog metadata.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id` | text | Internal stable id. |
| `uuid` | text | External stable uuid. |
| `tenant_id` | text | Required tenant scope. |
| `organization_id` | text | Optional organization scope; `0` or null policy to be finalized. |
| `game_code` | text | Unique per tenant/organization. |
| `title` | text | Display title. |
| `summary` | text nullable | Short display summary. |
| `genre` | text nullable | Catalog filter. |
| `cover_media` | json nullable | Drive/media reference, not raw provider URL ownership. |
| `status` | text | `draft`, `reviewing`, `published`, `disabled`, `archived`. |
| `visibility` | text | `public`, `private`, `invite_only`, `operator_only`. |
| `default_mode_id` | text nullable | Optional mode reference. |
| `sort_order` | integer | Catalog ordering. |
| `created_at`, `created_by`, `updated_at`, `updated_by`, `version`, `deleted_at`, `deleted_by` | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, organization_id, game_code)`.
- List: `(tenant_id, organization_id, status, sort_order, updated_at)`.
- Search/filter: `(tenant_id, status, genre, sort_order)`.

### 5.2 `game_mode`

Purpose: playable mode under a game, such as casual, ranked, solo, party, or event mode.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `game_id` | text | Owning game. |
| `mode_code` | text | Unique under game. |
| `title` | text | Display title. |
| `status` | text | `draft`, `active`, `disabled`, `archived`. |
| `min_players` | integer | Required; database and service enforce `>= 1`. |
| `max_players` | integer | Required; database and service enforce `min_players..64`. |
| `team_size` | integer nullable | Team-based mode support; when present, database and service enforce `1..max_players`. |
| `ruleset_id` | text nullable | Active rule set. |
| `matchmaking_enabled` | boolean | Whether ticket matching is allowed. |
| `room_enabled` | boolean | Whether manual rooms are allowed. |
| `leaderboard_enabled` | boolean | Whether rankings are available. |
| `settings` | json nullable | Typed mode settings; schema governed by ruleset. |
| standard audit columns | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, game_id, mode_code)`.
- List: `(tenant_id, game_id, status, sort_order)`.

### 5.3 `game_ruleset`

Purpose: versioned rule parameters for room, match, scoring, leaderboard, and settlement policies.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `game_id` | text | Owning game. |
| `mode_id` | text nullable | Optional mode-specific rule set. |
| `ruleset_code` | text | Stable code. |
| `version_no` | integer | Version number. |
| `status` | text | `draft`, `active`, `deprecated`, `archived`. |
| `config_schema` | json nullable | Parameter schema. |
| `config_values` | json | Parameter values. |
| `activated_at` | text nullable | Activation time. |
| standard audit columns | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, game_id, mode_id, ruleset_code, version_no)`.
- Lookup: `(tenant_id, game_id, mode_id, status)`.

### 5.4 `game_room`

Purpose: room aggregate root.

Current baseline columns and implemented service semantics:

| Column | Type | Notes |
| --- | --- | --- |
| `mode_id` | text nullable | Mode for the room. |
| `ruleset_id` | text nullable | Rules version snapshot. |
| `host_user_id` | text | Room owner/host IAM user id. |
| `visibility` | text | Current service values: `public`, `private`. |
| `join_policy` | text | Current service values: `open`, `invite`, `password`. |
| `room_password_hash` | text nullable | Reserved for password policy; never plaintext. |
| `max_players`, `current_players` | integer | Occupancy projection; service, OpenAPI, PostgreSQL, and SQLite enforce `max_players` in `1..64` and `current_players <= max_players`. |
| `status` | text | Current service values: `open`, `in_progress`, `closed`. |
| `opened_at`, `started_at`, `completed_at`, `closed_at`, `expires_at` | text nullable | State timestamps. |
| `metadata` | json nullable | Non-critical display/runtime metadata. |
| `version` | bigint | Optimistic concurrency for room commands. |

Indexes:

- Unique: `(tenant_id, room_code)`.
- List: `(tenant_id, game_id, mode_id, status, updated_at)`.
- Expiry: `(status, expires_at)`.

### 5.5 `game_room_seat`

Purpose: participant/seat state inside a room.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `room_id` | text | Owning room. |
| `seat_no` | integer | Stable seat index. |
| `team_no` | integer nullable | Team grouping. |
| `user_id` | text nullable | IAM user id when occupied. |
| `display_name_snapshot` | text nullable | UX snapshot. |
| `status` | text | Current service values: `empty`, `reserved`, `joined`, `ready`, `playing`, `left`. |
| `joined_at`, `ready_at`, `left_at` | text nullable | State timestamps. |
| `version` | integer | Optimistic concurrency. |

Indexes:

- Unique: `(tenant_id, room_id, seat_no)`.
- Unique active player seat candidate: `(tenant_id, room_id, user_id)` where active.
- List: `(tenant_id, room_id, status)`.

### 5.6 `game_match_ticket`

Purpose: matchmaking request.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `ticket_code` | text | External ticket id. |
| `game_id`, `mode_id`, `ruleset_id` | text | Match context. |
| `user_id` | text | Requesting player. |
| `party_id` | text nullable | Future party integration. |
| `status` | text | `queued`, `matching`, `matched`, `cancelled`, `timeout`, `failed`. |
| `priority` | integer | Queue priority. |
| `match_attributes` | json nullable | Rating/region/platform attributes. |
| `idempotency_key` | text | Required for create/retry. |
| `queued_at`, `matched_at`, `cancelled_at`, `expires_at` | text nullable | State timestamps. |

Indexes:

- Unique: `(tenant_id, idempotency_key)`.
- Queue: `(tenant_id, game_id, mode_id, status, priority, queued_at)`.
- Player active ticket: `(tenant_id, user_id, status)`.

### 5.7 `game_match_result`

Purpose: durable output of a matching attempt.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Event identity. |
| `game_id`, `mode_id`, `ruleset_id` | text | Match context. |
| `status` | text | `created`, `accepted`, `session_created`, `expired`, `failed`. |
| `ticket_ids` | json | Matched ticket list. |
| `participant_snapshot` | json | Player/team snapshot. |
| `session_id` | text nullable | Created session. |
| `created_at`, `accepted_at`, `expires_at` | text | State timestamps. |

Indexes:

- Lookup: `(tenant_id, game_id, mode_id, status, created_at)`.

### 5.8 `game_session`

Purpose: gameplay session lifecycle and result boundary.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `session_code` | text | External game-session id. |
| `game_id`, `mode_id`, `ruleset_id` | text | Session context. |
| `room_id` | text nullable | Source room. |
| `match_result_id` | text nullable | Source match. |
| `server_id` | text nullable | Game server/runtime identifier. |
| `status` | text | `created`, `starting`, `running`, `result_submitted`, `settling`, `completed`, `failed`, `voided`, `disputed`. |
| `started_at`, `ended_at`, `completed_at`, `voided_at` | text nullable | State timestamps. |
| `result_version` | integer | Result sequence/version. |
| `metadata` | json nullable | Non-critical runtime metadata. |
| standard audit columns | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, session_code)`.
- List: `(tenant_id, game_id, mode_id, status, updated_at)`.
- Room source: `(tenant_id, room_id)`.

### 5.9 `game_session_participant`

Purpose: participant snapshot and final per-player result.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Relation identity. |
| `session_id` | text | Owning session. |
| `user_id` | text | IAM user id. |
| `team_no` | integer nullable | Team grouping. |
| `display_name_snapshot` | text nullable | UX snapshot. |
| `status` | text | `joined`, `playing`, `completed`, `left`, `forfeited`, `disqualified`. |
| `score_delta` | bigint | Int64 serialized as string in APIs. |
| `rank_no` | integer nullable | Session-local rank. |
| `result_payload` | json nullable | Result details. |

Indexes:

- Unique: `(tenant_id, session_id, user_id)`.
- Player history: `(tenant_id, user_id, created_at)`.

### 5.10 `game_session_result`

Purpose: raw submitted result and validation state.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Event identity. |
| `session_id` | text | Owning session. |
| `source_type` | text | `game_server`, `operator`, `system`, `test`. |
| `source_id` | text nullable | Server/client/source identifier. |
| `idempotency_key` | text | Required. |
| `payload_hash` | text | Replay and immutability check. |
| `signature_status` | text | `not_required`, `verified`, `failed`. |
| `validation_status` | text | `pending`, `accepted`, `rejected`, `voided`. |
| `result_payload` | json | Submitted result. |
| `received_at`, `validated_at` | text | State timestamps. |
| `rejection_reason` | text nullable | Validation failure reason. |

Indexes:

- Unique: `(tenant_id, session_id, idempotency_key)`.
- Lookup: `(tenant_id, session_id, validation_status, received_at)`.

### 5.11 `game_score_event`

Purpose: validated score event from gameplay, operator correction, mission, or system source.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Event identity. |
| `game_id`, `mode_id`, `season_id` | text nullable | Score context. |
| `session_id`, `session_result_id` | text nullable | Source session/result. |
| `user_id` | text | Scored player. |
| `event_type` | text | `session_result`, `operator_adjustment`, `system_adjustment`, `achievement`, `mission`. |
| `score_delta` | bigint | Signed delta. |
| `score_absolute` | bigint nullable | Absolute score when event sets a value. |
| `reason_code` | text | Machine-readable reason. |
| `idempotency_key` | text | Required. |
| `event_payload` | json nullable | Source details. |
| `created_at` | text | Event time. |

Indexes:

- Unique: `(tenant_id, idempotency_key)`.
- Player timeline: `(tenant_id, user_id, game_id, created_at)`.
- Session source: `(tenant_id, session_id)`.

### 5.12 `game_point_ledger`

Purpose: append-only game point ledger.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Ledger identity. |
| `ledger_account_id` | text | Derived account key for game/mode/season/player. |
| `game_id`, `mode_id`, `season_id` | text nullable | Ledger context. |
| `user_id` | text | IAM user id. |
| `direction` | text | Current service accepts `credit` and `debit`. Correction semantics use `reason_code` plus one of those movement directions until the correction API is added. |
| `points_delta` | bigint | Positive movement amount; the applied sign is derived from `direction`. |
| `points_after` | bigint nullable | Projection value after apply. |
| `source_event_id` | text | Score event id. |
| `reason_code` | text | Reason for audit. |
| `idempotency_key` | text | Required. |
| `created_at` | text | Ledger time. |

Indexes:

- Unique: `(tenant_id, idempotency_key)`.
- Account timeline: `(tenant_id, ledger_account_id, created_at)`.
- Player/game: `(tenant_id, user_id, game_id, mode_id, season_id)`.

### 5.13 `game_point_balance`

Purpose: current point/rating projection.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Projection identity. |
| `ledger_account_id` | text | Unique account key. |
| `game_id`, `mode_id`, `season_id` | text nullable | Context. |
| `user_id` | text | IAM user id. |
| `points` | bigint | Current points. |
| `version` | integer | Projection version. |
| `last_ledger_id` | text | Latest applied ledger entry. |
| `updated_at` | text | Projection update time. |

Indexes:

- Unique: `(tenant_id, ledger_account_id)`.
- Ranking input: `(tenant_id, game_id, mode_id, season_id, points DESC, updated_at)`.

### 5.14 `game_leaderboard_config`

Purpose: leaderboard definition.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Config identity. |
| `game_id`, `mode_id`, `season_id` | text nullable | Scope. |
| `leaderboard_code` | text | Stable code. |
| `title` | text | Display title. |
| `status` | text | `draft`, `active`, `frozen`, `archived`. |
| `ranking_metric` | text | `points`, `score`, `wins`, future metrics. |
| `ranking_order` | text | `desc`, `asc`. |
| `tie_breaker` | text | `updated_at`, `first_reached`, `session_count`, etc. |
| `reset_policy` | text | `never`, `season`, `daily`, `weekly`, `monthly`. |
| `settings` | json nullable | Advanced ranking policy. |
| standard audit columns | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, game_id, mode_id, season_id, leaderboard_code)`.
- List: `(tenant_id, game_id, mode_id, status)`.

### 5.15 `game_leaderboard_entry`

Purpose: rank projection.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Projection identity. |
| `leaderboard_id` | text | Config id. |
| `game_id`, `mode_id`, `season_id` | text nullable | Denormalized query context. |
| `user_id` | text | IAM user id. |
| `display_name_snapshot` | text nullable | UX snapshot. |
| `score_value` | bigint | Ranking value. |
| `rank_no` | integer | Current rank. |
| `tie_breaker_value` | text nullable | Deterministic tie break value. |
| `last_ledger_id` | text nullable | Source ledger. |
| `updated_at` | text | Projection time. |

Indexes:

- Unique: `(tenant_id, leaderboard_id, user_id)`.
- Rank query: `(tenant_id, leaderboard_id, rank_no)`.
- Score query: `(tenant_id, leaderboard_id, score_value DESC, updated_at)`.
- My rank: `(tenant_id, user_id, leaderboard_id)`.

### 5.16 `game_season`

Purpose: season time window and lifecycle.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `game_id`, `mode_id` | text nullable | Scope. |
| `season_code` | text | Stable code. |
| `title` | text | Display title. |
| `status` | text | `draft`, `scheduled`, `active`, `settling`, `completed`, `archived`. |
| `starts_at`, `ends_at` | text | Window. |
| `settled_at` | text nullable | Settlement completion. |
| `settings` | json nullable | Reset/reward config. |
| standard audit columns | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, game_id, mode_id, season_code)`.
- Active lookup: `(tenant_id, game_id, mode_id, status, starts_at, ends_at)`.

### 5.17 `game_settlement_job`

Purpose: async settlement orchestration.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Job identity. |
| `session_id`, `session_result_id` | text | Settlement source. |
| `status` | text | `pending`, `running`, `succeeded`, `failed`, `cancelled`, `retrying`. |
| `attempt_count` | integer | Retry count. |
| `idempotency_key` | text | Required. |
| `error_code`, `error_detail` | text nullable | Last failure. |
| `job_payload` | json | Settlement inputs and computed actions. |
| `created_at`, `started_at`, `completed_at`, `next_retry_at` | text nullable | Job timestamps. |

Indexes:

- Unique: `(tenant_id, idempotency_key)`.
- Worker queue: `(status, next_retry_at, created_at)`.
- Session lookup: `(tenant_id, session_id)`.

### 5.18 `game_reward_intent`

Purpose: external reward grant intent.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Intent identity. |
| `settlement_job_id` | text | Source settlement. |
| `user_id` | text | Recipient. |
| `reward_type` | text | `points`, `wallet_credit`, `coupon`, `item`, `entitlement`, etc. |
| `external_owner` | text | `game`, `commerce`, `wallet`, `inventory`, etc. |
| `external_reference_id` | text nullable | Downstream id after execution. |
| `intent_payload` | json | Reward details. |
| `status` | text | `pending`, `submitted`, `succeeded`, `failed`, `cancelled`. |
| `idempotency_key` | text | Required. |
| `created_at`, `submitted_at`, `completed_at` | text nullable | State timestamps. |

Indexes:

- Unique: `(tenant_id, idempotency_key)`.
- Recipient: `(tenant_id, user_id, created_at)`.
- Worker: `(status, created_at)`.

### 5.19 `game_engine_event`

Purpose: game engine outbox event.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Event identity. |
| `event_type` | text | Example: `game.session.completed`. |
| `aggregate_type` | text | Source aggregate type. |
| `aggregate_id` | text | Source aggregate id. |
| `idempotency_key` | text | Required for replay-safe append. |
| `event_payload` | json | Event body. |
| `status` | text | `pending`, `published`, `failed`, `dead_letter`. |
| `trace_id` | text | Request trace. |
| `created_at`, `published_at`, `next_retry_at` | text nullable | Delivery timestamps. |

Indexes:

- Outbox: `(status, next_retry_at, created_at)`.
- Aggregate: `(tenant_id, aggregate_type, aggregate_id, created_at)`.

### 5.20 `game_webhook_subscription`

Purpose: webhook endpoint registration.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Tenant entity identity. |
| `game_id` | text nullable | Optional game scope. |
| `subscription_code` | text | Stable code. |
| `target_url` | text | Endpoint; secrets stored separately or encrypted according to security standards. |
| `event_types` | json | Subscribed event types. |
| `status` | text | `active`, `disabled`, `archived`. |
| `secret_ref` | text nullable | Secret reference, not plaintext. |
| standard audit columns | standard | Audit/lifecycle. |

Indexes:

- Unique: `(tenant_id, subscription_code)`.
- Dispatch lookup: `(tenant_id, game_id, status)`.

### 5.21 `game_webhook_delivery`

Purpose: webhook delivery attempts.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Delivery identity. |
| `event_id` | text | Source event. |
| `subscription_id` | text | Target subscription. |
| `status` | text | `pending`, `delivering`, `succeeded`, `failed`, `dead_letter`. |
| `attempt_count` | integer | Retry count. |
| `last_http_status` | integer nullable | Last response. |
| `last_error` | text nullable | Last failure. |
| `idempotency_key` | text | Delivery idempotency. |
| `created_at`, `delivered_at`, `next_retry_at` | text nullable | Delivery timestamps. |

Indexes:

- Unique: `(tenant_id, idempotency_key)`.
- Worker: `(status, next_retry_at, created_at)`.

### 5.22 `game_audit_record`

Purpose: operator/system audit record.

Key columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id`, `uuid`, `tenant_id`, `organization_id` | standard | Audit identity. |
| `actor_type` | text | `user`, `operator`, `system`, `server`, `job`. |
| `actor_id` | text nullable | IAM user or system id. |
| `action` | text | Machine-readable action. |
| `target_type` | text | Aggregate type. |
| `target_id` | text | Aggregate id. |
| `reason_code` | text nullable | Required for corrections/voids. |
| `before_snapshot` | json nullable | Before state when safe. |
| `after_snapshot` | json nullable | After state when safe. |
| `trace_id` | text | Trace id. |
| `created_at` | text | Audit time. |

Indexes:

- Target timeline: `(tenant_id, target_type, target_id, created_at)`.
- Actor timeline: `(tenant_id, actor_type, actor_id, created_at)`.
- Action search: `(tenant_id, action, created_at)`.

## 6. Leaderboard Baseline Decision

The executable database baseline uses a pre-GA clean baseline. The legacy single-table leaderboard
shape is removed instead of carried as a compatibility table.

Current storage is split by responsibility:

1. `game_leaderboard_config` stores leaderboard scope, metric, ordering, tie breaker, reset policy,
   status, and settings.
2. `game_leaderboard_entry` stores per-player rank projection rows for fast list and "my rank"
   queries.
3. Point and score history remain reconstructable from `game_score_event`, `game_point_ledger`, and
   `game_point_balance`; leaderboard entries are projections, not source-of-truth facts.

Implemented repository behavior:

- `game_match_ticket` supports exact idempotent replay on `(tenant_id, idempotency_key)`,
  conflicting-payload rejection, queued ticket cancellation, retrieve by id/code, tenant-scoped
  ticket list, and SQL-bounded queue pagination sorted by `priority DESC, queued_at ASC`.
- `game_session` and `game_session_participant` are created transactionally with participant
  snapshots. Session start updates status, `server_id`, `started_at`, and optimistic version.
- `game_session_result` supports exact idempotent replay on
  `(tenant_id, session_id, idempotency_key)`, conflicting-payload rejection, immutable payload hash,
  and result-version projection update on the owning session.
- `game_settlement_job` supports idempotent creation on `(tenant_id, idempotency_key)`,
  start-from-pending/retrying, failure-to-retrying or failed transitions, successful completion, and
  SQL-bounded due-job pagination for workers.
- `game_reward_intent` supports idempotent creation and validates that the settlement job exists.
  It records external grant intent only; game engine services do not write wallet, commerce,
  inventory, or entitlement-owned tables directly.
- `game_engine_event` supports idempotent outbox append, pending/failed due-event pagination,
  published marking, retry failure marking, and dead-letter marking.
- `game_audit_record` is append-only and supports tenant-scoped search by target, actor, action,
  and created time using SQL filters and bounded pages.
- `game_point_ledger` is append-only at the service contract. Duplicate `tenant_id +
  idempotency_key` requests return the existing ledger entry when the payload matches.
- Conflicting payloads for the same idempotency key are rejected instead of overwriting ledger or
  balance state.
- `game_point_balance` is updated as a projection in the same repository operation and records the
  latest applied ledger id.
- `game_leaderboard_entry` supports upsert and full rebuild for a leaderboard scope. Rank numbers are
  recalculated by `score_value DESC`, `recorded_at ASC`, then `id ASC`.

If this application later reaches GA with production data, future leaderboard schema changes must use
expand/backfill/verify/shrink migrations instead of rewriting the baseline.

## 7. Standard Status Vocabularies

| Aggregate | Status values |
| --- | --- |
| Game | `draft`, `reviewing`, `published`, `disabled`, `archived` |
| Mode | `draft`, `active`, `disabled`, `archived` |
| RuleSet | `draft`, `active`, `deprecated`, `archived` |
| Room | Current service values: `open`, `in_progress`, `closed`; future values require service/API contract expansion. |
| Seat | Current service values: `empty`, `reserved`, `joined`, `ready`, `playing`, `left`; future removal/kick semantics require service/API contract expansion. |
| MatchTicket | Current repository values: `queued`, `cancelled`; planned matcher values: `matching`, `matched`, `timeout`, `failed`. |
| MatchResult | `created`, `accepted`, `session_created`, `expired`, `failed` |
| Session | Current repository values: `created`, `started`, `completed`; planned workflow values: `starting`, `running`, `result_submitted`, `settling`, `failed`, `voided`, `disputed`. |
| SessionResult | Current repository value: `validated`; planned validation values: `pending`, `accepted`, `rejected`, `voided`. |
| Leaderboard | `draft`, `active`, `frozen`, `archived` |
| Season | `draft`, `scheduled`, `active`, `settling`, `completed`, `archived` |
| SettlementJob | `pending`, `running`, `succeeded`, `failed`, `cancelled`, `retrying` |
| RewardIntent | `pending`, `submitted`, `succeeded`, `failed`, `cancelled` |
| EngineEvent | `pending`, `published`, `failed`, `dead_letter` |
| WebhookDelivery | `pending`, `delivering`, `succeeded`, `failed`, `dead_letter` |

## 8. Implementation Sequence

Implementation sequence:

1. Keep database contract and table registry generated from the PostgreSQL baseline with declared
   PostgreSQL and SQLite engines.
2. Keep PostgreSQL and SQLite baselines aligned for cloud, standalone, and local packaging parity.
3. Add repository tests around list pagination, unique constraints, and idempotency.
4. Add Rust repository/service modules in bounded increments.
5. Add route manifests and OpenAPI operations.
6. Regenerate SDKs and update composed facades.
7. Add PC service facade and backend operations views.
8. Run database, API, SDK, Rust, and repository verification.

## 9. Verification Commands

Design-only verification:

```bash
pnpm run typecheck
```

Implementation verification after schema/API changes:

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run api:check
pnpm run sdk:check
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
pnpm run verify
cargo test --workspace
```

Additional SDKWork standard validators must run when touching list/search APIs, response envelopes,
or SDK consumers, as specified in `AGENTS.md`.
