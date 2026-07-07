use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_session_service::{
    CreateGameSessionCommand, GameSessionError, GameSessionItem, GameSessionParticipantItem,
    GameSessionRepository, GameSessionResult, GameSessionResultItem, StartGameSessionCommand,
    SubmitSessionResultCommand,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;

#[derive(Clone, Default)]
pub struct InMemoryGameSessionRepository {
    store: Arc<RwLock<SessionStore>>,
}

#[derive(Default)]
struct SessionStore {
    sessions: Vec<StoredSession>,
    participants: Vec<StoredParticipant>,
    results: Vec<StoredResult>,
}

#[derive(Clone)]
struct StoredSession {
    tenant_id: String,
    item: GameSessionItem,
}

#[derive(Clone)]
struct StoredParticipant {
    tenant_id: String,
    item: GameSessionParticipantItem,
}

#[derive(Clone)]
struct StoredResult {
    tenant_id: String,
    item: GameSessionResultItem,
}

#[async_trait]
impl GameSessionRepository for InMemoryGameSessionRepository {
    async fn create_session(
        &self,
        tenant_id: &str,
        command: &CreateGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        if store.sessions.iter().any(|stored| {
            stored.tenant_id == tenant_id && stored.item.session_code == command.session_code
        }) {
            return Err(GameSessionError::conflict(
                "session_code already exists for tenant",
            ));
        }
        let id = uuid();
        let item = GameSessionItem {
            id: id.clone(),
            session_code: command.session_code.clone(),
            game_id: command.game_id.clone(),
            mode_id: command.mode_id.clone(),
            ruleset_id: command.ruleset_id.clone(),
            room_id: command.room_id.clone(),
            match_result_id: command.match_result_id.clone(),
            server_id: command.server_id.clone(),
            status: "created".into(),
            started_at: None,
            ended_at: None,
            completed_at: None,
            result_version: 0,
            metadata: command.metadata.clone(),
            version: 0,
        };
        store.sessions.push(StoredSession {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });

        for participant in &command.participants {
            store.participants.push(StoredParticipant {
                tenant_id: tenant_id.into(),
                item: GameSessionParticipantItem {
                    id: uuid(),
                    session_id: id.clone(),
                    user_id: participant.user_id.clone(),
                    team_no: participant.team_no,
                    display_name_snapshot: participant.display_name_snapshot.clone(),
                    status: "joined".into(),
                    score_delta: 0,
                    rank_no: None,
                    result_payload: serde_json::json!({}),
                    version: 0,
                },
            });
        }
        Ok(item)
    }

    async fn get_session(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<GameSessionItem> {
        let store = self.store.read().map_err(lock_error)?;
        store
            .sessions
            .iter()
            .find(|stored| {
                stored.tenant_id == tenant_id
                    && (stored.item.id == session_id || stored.item.session_code == session_id)
            })
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameSessionError::not_found("session not found"))
    }

    async fn list_participants(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<Vec<GameSessionParticipantItem>> {
        let store = self.store.read().map_err(lock_error)?;
        let mut participants: Vec<GameSessionParticipantItem> = store
            .participants
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id && stored.item.session_id == session_id)
            .map(|stored| stored.item.clone())
            .collect();
        participants.sort_by(|left, right| {
            left.team_no
                .unwrap_or_default()
                .cmp(&right.team_no.unwrap_or_default())
                .then_with(|| left.user_id.cmp(&right.user_id))
        });
        Ok(participants)
    }

    async fn start_session(
        &self,
        tenant_id: &str,
        command: &StartGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let stored = store
            .sessions
            .iter_mut()
            .find(|stored| stored.tenant_id == tenant_id && stored.item.id == command.session_id)
            .ok_or_else(|| GameSessionError::not_found("session not found"))?;
        if let Some(expected_version) = command.expected_version {
            if stored.item.version != expected_version {
                return Err(GameSessionError::conflict("session version has changed"));
            }
        }
        stored.item.status = "started".into();
        if let Some(server_id) = &command.server_id {
            stored.item.server_id = Some(server_id.clone());
        }
        stored.item.started_at = Some(now().to_rfc3339());
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn submit_result(
        &self,
        tenant_id: &str,
        command: &SubmitSessionResultCommand,
    ) -> GameSessionResult<GameSessionResultItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        if let Some(existing) = store.results.iter().find(|stored| {
            stored.tenant_id == tenant_id
                && stored.item.session_id == command.session_id
                && stored.item.idempotency_key == command.idempotency_key
        }) {
            ensure_idempotent_replay(&existing.item, command)?;
            return Ok(existing.item.clone());
        }

        let session = store
            .sessions
            .iter_mut()
            .find(|stored| stored.tenant_id == tenant_id && stored.item.id == command.session_id)
            .ok_or_else(|| GameSessionError::not_found("session not found"))?;

        let timestamp = now().to_rfc3339();
        let item = GameSessionResultItem {
            id: uuid(),
            session_id: command.session_id.clone(),
            source_type: command.source_type.clone(),
            source_id: command.source_id.clone(),
            idempotency_key: command.idempotency_key.clone(),
            payload_hash: command.payload_hash.clone(),
            signature_status: "not_required".into(),
            validation_status: "validated".into(),
            result_payload: command.result_payload.clone(),
            received_at: timestamp.clone(),
            validated_at: Some(timestamp.clone()),
            rejection_reason: None,
            version: 0,
        };
        session.item.result_version += 1;
        session.item.status = "completed".into();
        session.item.completed_at = Some(timestamp);
        session.item.version += 1;
        store.results.push(StoredResult {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }
}

fn ensure_idempotent_replay(
    existing: &GameSessionResultItem,
    command: &SubmitSessionResultCommand,
) -> GameSessionResult<()> {
    let same_payload = existing.source_type == command.source_type
        && existing.source_id == command.source_id
        && existing.payload_hash == command.payload_hash
        && existing.result_payload == command.result_payload;
    if !same_payload {
        return Err(GameSessionError::conflict(
            "idempotency_key already belongs to a different session result payload",
        ));
    }
    Ok(())
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameSessionError {
    GameSessionError::invalid("session repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use sdkwork_game_session_service::{
        CreateGameSessionCommand, CreateGameSessionParticipant, GameSessionRepository,
        SubmitSessionResultCommand,
    };
    use serde_json::json;

    use super::InMemoryGameSessionRepository;

    fn create_command() -> CreateGameSessionCommand {
        CreateGameSessionCommand {
            session_code: "S-1".into(),
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            ruleset_id: Some("ruleset-standard".into()),
            room_id: Some("room-1".into()),
            match_result_id: None,
            server_id: Some("server-1".into()),
            created_by: Some("user-host".into()),
            metadata: json!({"map": "classic"}),
            participants: vec![CreateGameSessionParticipant {
                user_id: "user-1".into(),
                team_no: Some(1),
                display_name_snapshot: Some("Player 1".into()),
            }],
        }
    }

    fn result_command(session_id: &str, hash: &str) -> SubmitSessionResultCommand {
        SubmitSessionResultCommand {
            session_id: session_id.into(),
            source_type: "game_server".into(),
            source_id: Some("server-1".into()),
            idempotency_key: "idem-result-1".into(),
            payload_hash: hash.into(),
            result_payload: json!({"winner": "user-1"}),
        }
    }

    #[tokio::test]
    async fn create_session_persists_participants() {
        let repository = InMemoryGameSessionRepository::default();
        let session = repository
            .create_session("100001", &create_command())
            .await
            .unwrap();
        let participants = repository
            .list_participants("100001", &session.id)
            .await
            .unwrap();

        assert_eq!(1, participants.len());
        assert_eq!("user-1", participants[0].user_id);
    }

    #[tokio::test]
    async fn submit_result_is_idempotent_and_conflict_checked() {
        let repository = InMemoryGameSessionRepository::default();
        let session = repository
            .create_session("100001", &create_command())
            .await
            .unwrap();

        let first = repository
            .submit_result("100001", &result_command(&session.id, "hash-1"))
            .await
            .unwrap();
        let replay = repository
            .submit_result("100001", &result_command(&session.id, "hash-1"))
            .await
            .unwrap();
        assert_eq!(first.id, replay.id);

        let error = repository
            .submit_result("100001", &result_command(&session.id, "hash-2"))
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }
}
