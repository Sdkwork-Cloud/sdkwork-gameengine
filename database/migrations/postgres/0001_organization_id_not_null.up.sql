-- sdkwork:migration
-- id: 0001_organization_id_not_null
-- engine: postgres
-- module: sdkwork-gameengine
-- purpose: Enforce organization_id NOT NULL DEFAULT on all tables in the
--   consolidated baseline. NULL rows (pre-standard data anomalies) are
--   backfilled with the platform sentinel before NOT NULL is set, and
--   NOT NULL columns without an explicit default receive the sentinel
--   default, keeping existing deployments consistent with fresh baseline
--   installs.
-- reversible: false
-- rollback: forward-fix (sentinel backfill is the canonical fix; NULL
--   organization rows are data anomalies)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

UPDATE game_catalog SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_catalog ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_catalog ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_mode SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_mode ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_mode ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_ruleset SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_ruleset ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_ruleset ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_room SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_room ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_room ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_room_seat SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_room_seat ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_room_seat ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_match_ticket SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_match_ticket ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_match_ticket ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_match_result SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_match_result ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_match_result ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_session SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_session ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_session ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_session_participant SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_session_participant ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_session_participant ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_session_result SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_session_result ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_session_result ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_score_event SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_score_event ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_score_event ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_point_ledger SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_point_ledger ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_point_ledger ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_point_balance SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_point_balance ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_point_balance ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_leaderboard_config SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_leaderboard_config ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_leaderboard_config ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_leaderboard_entry SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_leaderboard_entry ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_leaderboard_entry ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_settlement_job SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_settlement_job ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_settlement_job ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_reward_intent SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_reward_intent ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_reward_intent ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_engine_event SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_engine_event ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_engine_event ALTER COLUMN organization_id SET NOT NULL;

UPDATE game_audit_record SET organization_id = '0' WHERE organization_id IS NULL;
ALTER TABLE game_audit_record ALTER COLUMN organization_id SET DEFAULT '0';
ALTER TABLE game_audit_record ALTER COLUMN organization_id SET NOT NULL;

COMMIT;
