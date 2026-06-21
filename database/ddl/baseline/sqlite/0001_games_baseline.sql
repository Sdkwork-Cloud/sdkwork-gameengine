-- SDKWork games database baseline (moduleId=games, prefix=game_)

CREATE TABLE IF NOT EXISTS game_catalog (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT,
  game_code TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT,
  genre TEXT,
  cover_url TEXT,
  status TEXT NOT NULL DEFAULT 'draft',
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

CREATE TABLE IF NOT EXISTS game_room (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT,
  game_id TEXT NOT NULL,
  room_code TEXT NOT NULL,
  max_players INTEGER NOT NULL DEFAULT 4,
  current_players INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'open',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (tenant_id, room_code)
);

CREATE TABLE IF NOT EXISTS game_leaderboard (
  id TEXT PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  tenant_id TEXT NOT NULL,
  organization_id TEXT,
  game_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  score INTEGER NOT NULL DEFAULT 0,
  rank_no INTEGER,
  recorded_at TEXT NOT NULL,
  UNIQUE (tenant_id, game_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_game_catalog_tenant_status ON game_catalog (tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_game_room_game_status ON game_room (game_id, status);
CREATE INDEX IF NOT EXISTS idx_game_leaderboard_game_score ON game_leaderboard (game_id, score DESC);
