-- SDKWork game engine consolidated initialization baseline (sqlite)
-- Application is in initialization state: this file is the full pre-GA schema snapshot.
-- Migrations are reserved for post-GA incremental schema changes.

-- baseline source: ddl/baseline/sqlite/0001_games_baseline.sql
-- SDKWork game engine database baseline (moduleId=games, prefix=game_)

CREATE TABLE IF NOT EXISTS game_catalog (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_code TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT,
  genre TEXT,
  cover_media TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'draft',
  visibility TEXT NOT NULL DEFAULT 'private',
  default_mode_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  deleted_at TEXT,
  deleted_by TEXT,
  UNIQUE (tenant_id, organization_id, game_code)
);

CREATE TABLE IF NOT EXISTS game_mode (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_id TEXT NOT NULL,
  mode_code TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft',
  min_players INTEGER NOT NULL DEFAULT 1,
  max_players INTEGER NOT NULL DEFAULT 1,
  team_size INTEGER,
  ruleset_id TEXT,
  matchmaking_enabled INTEGER NOT NULL DEFAULT 0,
  room_enabled INTEGER NOT NULL DEFAULT 1,
  leaderboard_enabled INTEGER NOT NULL DEFAULT 1,
  settings TEXT NOT NULL DEFAULT '{}',
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  deleted_at TEXT,
  deleted_by TEXT,
  UNIQUE (tenant_id, game_id, mode_code),
  CHECK (min_players >= 1),
  CHECK (max_players >= min_players)
);

CREATE TABLE IF NOT EXISTS game_ruleset (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_id TEXT NOT NULL,
  mode_id TEXT,
  ruleset_code TEXT NOT NULL,
  version_no INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'draft',
  config_schema TEXT NOT NULL DEFAULT '{}',
  config_values TEXT NOT NULL DEFAULT '{}',
  activated_at TEXT,
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  deleted_at TEXT,
  deleted_by TEXT,
  UNIQUE (tenant_id, game_id, mode_id, ruleset_code, version_no)
);

CREATE TABLE IF NOT EXISTS game_room (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_id TEXT NOT NULL,
  mode_id TEXT,
  ruleset_id TEXT,
  room_code TEXT NOT NULL,
  host_user_id TEXT,
  visibility TEXT NOT NULL DEFAULT 'public',
  join_policy TEXT NOT NULL DEFAULT 'open',
  room_password_hash TEXT,
  max_players INTEGER NOT NULL DEFAULT 4,
  current_players INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'open',
  opened_at TEXT,
  started_at TEXT,
  completed_at TEXT,
  closed_at TEXT,
  expires_at TEXT,
  metadata TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  deleted_at TEXT,
  deleted_by TEXT,
  UNIQUE (tenant_id, room_code),
  CHECK (max_players >= 1),
  CHECK (current_players >= 0),
  CHECK (current_players <= max_players)
);

CREATE TABLE IF NOT EXISTS game_room_seat (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  room_id TEXT NOT NULL,
  seat_no INTEGER NOT NULL,
  team_no INTEGER,
  user_id TEXT,
  display_name_snapshot TEXT,
  status TEXT NOT NULL DEFAULT 'empty',
  joined_at TEXT,
  ready_at TEXT,
  left_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, room_id, seat_no),
  CHECK (seat_no >= 1)
);

CREATE TABLE IF NOT EXISTS game_match_ticket (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  ticket_code TEXT NOT NULL,
  game_id TEXT NOT NULL,
  mode_id TEXT,
  ruleset_id TEXT,
  user_id TEXT NOT NULL,
  party_id TEXT,
  status TEXT NOT NULL DEFAULT 'queued',
  priority INTEGER NOT NULL DEFAULT 0,
  match_attributes TEXT NOT NULL DEFAULT '{}',
  idempotency_key TEXT NOT NULL,
  queued_at TEXT NOT NULL,
  matched_at TEXT,
  cancelled_at TEXT,
  expires_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, idempotency_key),
  UNIQUE (tenant_id, ticket_code)
);

CREATE TABLE IF NOT EXISTS game_match_result (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_id TEXT NOT NULL,
  mode_id TEXT,
  ruleset_id TEXT,
  status TEXT NOT NULL DEFAULT 'created',
  ticket_ids TEXT NOT NULL DEFAULT '[]',
  participant_snapshot TEXT NOT NULL DEFAULT '[]',
  session_id TEXT,
  created_at TEXT NOT NULL,
  accepted_at TEXT,
  expires_at TEXT,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS game_session (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  session_code TEXT NOT NULL,
  game_id TEXT NOT NULL,
  mode_id TEXT,
  ruleset_id TEXT,
  room_id TEXT,
  match_result_id TEXT,
  server_id TEXT,
  status TEXT NOT NULL DEFAULT 'created',
  started_at TEXT,
  ended_at TEXT,
  completed_at TEXT,
  voided_at TEXT,
  result_version INTEGER NOT NULL DEFAULT 0,
  metadata TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  deleted_at TEXT,
  deleted_by TEXT,
  UNIQUE (tenant_id, session_code)
);

CREATE TABLE IF NOT EXISTS game_session_participant (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  session_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  team_no INTEGER,
  display_name_snapshot TEXT,
  status TEXT NOT NULL DEFAULT 'joined',
  score_delta INTEGER NOT NULL DEFAULT 0,
  rank_no INTEGER,
  result_payload TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, session_id, user_id)
);

CREATE TABLE IF NOT EXISTS game_session_result (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  session_id TEXT NOT NULL,
  source_type TEXT NOT NULL,
  source_id TEXT,
  idempotency_key TEXT NOT NULL,
  payload_hash TEXT NOT NULL,
  signature_status TEXT NOT NULL DEFAULT 'not_required',
  validation_status TEXT NOT NULL DEFAULT 'pending',
  result_payload TEXT NOT NULL DEFAULT '{}',
  received_at TEXT NOT NULL,
  validated_at TEXT,
  rejection_reason TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, session_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS game_score_event (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_id TEXT NOT NULL,
  mode_id TEXT,
  season_id TEXT,
  session_id TEXT,
  session_result_id TEXT,
  user_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'accepted',
  score_delta INTEGER NOT NULL DEFAULT 0,
  score_absolute INTEGER,
  reason_code TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  event_payload TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS game_point_ledger (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  ledger_account_id TEXT NOT NULL,
  game_id TEXT NOT NULL,
  mode_id TEXT,
  season_id TEXT,
  user_id TEXT NOT NULL,
  direction TEXT NOT NULL,
  points_delta INTEGER NOT NULL DEFAULT 0,
  points_after INTEGER,
  source_event_id TEXT NOT NULL,
  reason_code TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS game_point_balance (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  ledger_account_id TEXT NOT NULL,
  game_id TEXT NOT NULL,
  mode_id TEXT,
  season_id TEXT,
  user_id TEXT NOT NULL,
  points INTEGER NOT NULL DEFAULT 0,
  last_ledger_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, ledger_account_id)
);

CREATE TABLE IF NOT EXISTS game_leaderboard_config (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  game_id TEXT NOT NULL,
  mode_id TEXT,
  season_id TEXT,
  leaderboard_code TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft',
  ranking_metric TEXT NOT NULL DEFAULT 'points',
  ranking_order TEXT NOT NULL DEFAULT 'desc',
  tie_breaker TEXT NOT NULL DEFAULT 'updated_at',
  reset_policy TEXT NOT NULL DEFAULT 'never',
  settings TEXT NOT NULL DEFAULT '{}',
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  created_by TEXT,
  updated_at TEXT NOT NULL,
  updated_by TEXT,
  version INTEGER NOT NULL DEFAULT 0,
  deleted_at TEXT,
  deleted_by TEXT
);

CREATE TABLE IF NOT EXISTS game_leaderboard_entry (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  leaderboard_id TEXT NOT NULL,
  game_id TEXT NOT NULL,
  mode_id TEXT,
  season_id TEXT,
  user_id TEXT NOT NULL,
  display_name_snapshot TEXT,
  score_value INTEGER NOT NULL DEFAULT 0,
  rank_no INTEGER,
  tie_breaker_value TEXT,
  last_ledger_id TEXT,
  recorded_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, leaderboard_id, user_id)
);

CREATE TABLE IF NOT EXISTS game_settlement_job (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  session_id TEXT NOT NULL,
  session_result_id TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  attempt_count INTEGER NOT NULL DEFAULT 0,
  idempotency_key TEXT NOT NULL,
  error_code TEXT,
  error_detail TEXT,
  job_payload TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  started_at TEXT,
  completed_at TEXT,
  next_retry_at TEXT,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS game_reward_intent (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  settlement_job_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  reward_type TEXT NOT NULL,
  external_owner TEXT NOT NULL,
  external_reference_id TEXT,
  intent_payload TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'pending',
  idempotency_key TEXT NOT NULL,
  created_at TEXT NOT NULL,
  submitted_at TEXT,
  completed_at TEXT,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS game_engine_event (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  event_type TEXT NOT NULL,
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  event_payload TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'pending',
  trace_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  published_at TEXT,
  next_retry_at TEXT,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS game_audit_record (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT NOT NULL DEFAULT '0',
  actor_type TEXT NOT NULL,
  actor_id TEXT,
  action TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id TEXT NOT NULL,
  reason_code TEXT,
  before_snapshot TEXT NOT NULL DEFAULT '{}',
  after_snapshot TEXT NOT NULL DEFAULT '{}',
  trace_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_game_catalog_tenant_status_sort
  ON game_catalog (tenant_id, organization_id, status, sort_order, updated_at);
CREATE INDEX IF NOT EXISTS idx_game_catalog_tenant_genre_sort
  ON game_catalog (tenant_id, status, genre, sort_order);

CREATE INDEX IF NOT EXISTS idx_game_mode_game_status_sort
  ON game_mode (tenant_id, game_id, status, sort_order);
CREATE INDEX IF NOT EXISTS idx_game_ruleset_lookup
  ON game_ruleset (tenant_id, game_id, mode_id, status);

CREATE INDEX IF NOT EXISTS idx_game_room_game_mode_status
  ON game_room (tenant_id, game_id, mode_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_game_room_expiry
  ON game_room (status, expires_at);

CREATE INDEX IF NOT EXISTS idx_game_room_seat_room_status
  ON game_room_seat (tenant_id, room_id, status);
CREATE UNIQUE INDEX IF NOT EXISTS uk_game_room_seat_active_user
  ON game_room_seat (tenant_id, room_id, user_id)
  WHERE user_id IS NOT NULL AND status IN ('reserved', 'joined', 'ready', 'playing');

CREATE INDEX IF NOT EXISTS idx_game_match_ticket_queue
  ON game_match_ticket (tenant_id, game_id, mode_id, status, priority DESC, queued_at);
CREATE INDEX IF NOT EXISTS idx_game_match_ticket_player_status
  ON game_match_ticket (tenant_id, user_id, status);
CREATE INDEX IF NOT EXISTS idx_game_match_ticket_expiry
  ON game_match_ticket (tenant_id, status, expires_at);

CREATE INDEX IF NOT EXISTS idx_game_match_result_lookup
  ON game_match_result (tenant_id, game_id, mode_id, status, created_at);

CREATE INDEX IF NOT EXISTS idx_game_session_list
  ON game_session (tenant_id, game_id, mode_id, status, updated_at);
CREATE INDEX IF NOT EXISTS idx_game_session_room
  ON game_session (tenant_id, room_id);
CREATE INDEX IF NOT EXISTS idx_game_session_match_result
  ON game_session (tenant_id, match_result_id);

CREATE INDEX IF NOT EXISTS idx_game_session_participant_player
  ON game_session_participant (tenant_id, user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_game_session_participant_session_status
  ON game_session_participant (tenant_id, session_id, status);

CREATE INDEX IF NOT EXISTS idx_game_session_result_lookup
  ON game_session_result (tenant_id, session_id, validation_status, received_at);

CREATE INDEX IF NOT EXISTS idx_game_score_event_player_timeline
  ON game_score_event (tenant_id, user_id, game_id, created_at);
CREATE INDEX IF NOT EXISTS idx_game_score_event_session
  ON game_score_event (tenant_id, session_id);

CREATE INDEX IF NOT EXISTS idx_game_point_ledger_account_timeline
  ON game_point_ledger (tenant_id, ledger_account_id, created_at);
CREATE INDEX IF NOT EXISTS idx_game_point_ledger_player_game
  ON game_point_ledger (tenant_id, user_id, game_id, mode_id, season_id);

CREATE INDEX IF NOT EXISTS idx_game_point_balance_ranking
  ON game_point_balance (tenant_id, game_id, mode_id, season_id, points DESC, updated_at);

CREATE UNIQUE INDEX IF NOT EXISTS uk_game_leaderboard_config_scope_code
  ON game_leaderboard_config (
    tenant_id,
    game_id,
    COALESCE(mode_id, ''),
    COALESCE(season_id, ''),
    leaderboard_code
  );
CREATE INDEX IF NOT EXISTS idx_game_leaderboard_config_list
  ON game_leaderboard_config (tenant_id, game_id, mode_id, status);

CREATE INDEX IF NOT EXISTS idx_game_leaderboard_entry_rank
  ON game_leaderboard_entry (tenant_id, leaderboard_id, rank_no);
CREATE INDEX IF NOT EXISTS idx_game_leaderboard_entry_score
  ON game_leaderboard_entry (tenant_id, leaderboard_id, score_value DESC, updated_at);
CREATE INDEX IF NOT EXISTS idx_game_leaderboard_entry_user
  ON game_leaderboard_entry (tenant_id, user_id, leaderboard_id);

CREATE INDEX IF NOT EXISTS idx_game_settlement_job_worker
  ON game_settlement_job (status, next_retry_at, created_at);
CREATE INDEX IF NOT EXISTS idx_game_settlement_job_session
  ON game_settlement_job (tenant_id, session_id);
CREATE INDEX IF NOT EXISTS idx_game_reward_intent_recipient
  ON game_reward_intent (tenant_id, user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_game_reward_intent_worker
  ON game_reward_intent (status, created_at);
CREATE INDEX IF NOT EXISTS idx_game_engine_event_outbox
  ON game_engine_event (status, next_retry_at, created_at);
CREATE INDEX IF NOT EXISTS idx_game_engine_event_aggregate
  ON game_engine_event (tenant_id, aggregate_type, aggregate_id, created_at);

CREATE INDEX IF NOT EXISTS idx_game_audit_record_target
  ON game_audit_record (tenant_id, target_type, target_id, created_at);
CREATE INDEX IF NOT EXISTS idx_game_audit_record_actor
  ON game_audit_record (tenant_id, actor_type, actor_id, created_at);
CREATE INDEX IF NOT EXISTS idx_game_audit_record_action
  ON game_audit_record (tenant_id, action, created_at);
