use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    CreateGameSessionCommand, GameSessionError, GameSessionItem, GameSessionParticipantItem,
    GameSessionResult, GameSessionResultItem, StartGameSessionCommand, SubmitSessionResultCommand,
};
use crate::ports::repository::GameSessionRepository;

pub struct GameSessionService<R> {
    repository: R,
}

impl<R> GameSessionService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameSessionService<R>
where
    R: GameSessionRepository,
{
    pub async fn create_session(
        &self,
        tenant_id: &str,
        command: CreateGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("session_code", &command.session_code)?;
        validate_required("game_id", &command.game_id)?;
        if command.participants.is_empty() {
            return Err(GameSessionError::invalid(
                "session participants must not be empty",
            ));
        }
        for participant in &command.participants {
            validate_required("participant.user_id", &participant.user_id)?;
        }
        self.repository.create_session(tenant_id, &command).await
    }

    pub async fn get_session(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<GameSessionItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("session_id", session_id)?;
        self.repository.get_session(tenant_id, session_id).await
    }

    pub async fn list_participants(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<Vec<GameSessionParticipantItem>> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("session_id", session_id)?;
        self.repository
            .list_participants(tenant_id, session_id)
            .await
    }

    pub async fn start_session(
        &self,
        tenant_id: &str,
        command: StartGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("session_id", &command.session_id)?;
        let session = self
            .repository
            .get_session(tenant_id, &command.session_id)
            .await?;
        if let Some(expected_version) = command.expected_version {
            if session.version != expected_version {
                return Err(GameSessionError::conflict("session version has changed"));
            }
        }
        if !matches!(session.status.as_str(), "created" | "starting") {
            return Err(GameSessionError::invalid(
                "session can only start from created or starting status",
            ));
        }
        self.repository.start_session(tenant_id, &command).await
    }

    pub async fn submit_result(
        &self,
        tenant_id: &str,
        command: SubmitSessionResultCommand,
    ) -> GameSessionResult<GameSessionResultItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("session_id", &command.session_id)?;
        validate_required("source_type", &command.source_type)?;
        validate_source_type(&command.source_type)?;
        validate_required("idempotency_key", &command.idempotency_key)?;
        validate_required("payload_hash", &command.payload_hash)?;
        if command.result_payload.is_null() {
            return Err(GameSessionError::invalid("result_payload must not be null"));
        }
        self.repository.submit_result(tenant_id, &command).await
    }
}

fn validate_required(field: &str, value: &str) -> GameSessionResult<()> {
    if is_blank(Some(value)) {
        return Err(GameSessionError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_source_type(source_type: &str) -> GameSessionResult<()> {
    if matches!(source_type, "game_server" | "operator" | "system" | "test") {
        return Ok(());
    }
    Err(GameSessionError::invalid(
        "session result source_type is not supported",
    ))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;
    use crate::domain::models::{
        CreateGameSessionParticipant, GameSessionError, GameSessionResultItem,
    };

    struct EmptyRepo;

    #[async_trait]
    impl GameSessionRepository for EmptyRepo {
        async fn create_session(
            &self,
            _tenant_id: &str,
            command: &CreateGameSessionCommand,
        ) -> GameSessionResult<GameSessionItem> {
            Ok(session_item(command))
        }

        async fn get_session(
            &self,
            _tenant_id: &str,
            _session_id: &str,
        ) -> GameSessionResult<GameSessionItem> {
            Err(GameSessionError::not_found("session not found"))
        }

        async fn list_participants(
            &self,
            _tenant_id: &str,
            _session_id: &str,
        ) -> GameSessionResult<Vec<GameSessionParticipantItem>> {
            Ok(vec![])
        }

        async fn start_session(
            &self,
            _tenant_id: &str,
            _command: &StartGameSessionCommand,
        ) -> GameSessionResult<GameSessionItem> {
            Err(GameSessionError::invalid("unexpected repository call"))
        }

        async fn submit_result(
            &self,
            _tenant_id: &str,
            command: &SubmitSessionResultCommand,
        ) -> GameSessionResult<GameSessionResultItem> {
            Ok(GameSessionResultItem {
                id: "result-1".into(),
                session_id: command.session_id.clone(),
                source_type: command.source_type.clone(),
                source_id: command.source_id.clone(),
                idempotency_key: command.idempotency_key.clone(),
                payload_hash: command.payload_hash.clone(),
                signature_status: "not_required".into(),
                validation_status: "validated".into(),
                result_payload: command.result_payload.clone(),
                received_at: "2026-07-07T00:00:00Z".into(),
                validated_at: Some("2026-07-07T00:00:00Z".into()),
                rejection_reason: None,
                version: 0,
            })
        }
    }

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

    fn session_item(command: &CreateGameSessionCommand) -> GameSessionItem {
        GameSessionItem {
            id: "session-1".into(),
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
        }
    }

    #[tokio::test]
    async fn create_session_rejects_empty_participants() {
        let service = GameSessionService::new(EmptyRepo);
        let mut command = create_command();
        command.participants.clear();

        let error = service
            .create_session("100001", command)
            .await
            .expect_err("empty participants must fail");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn submit_result_rejects_unsupported_source_type() {
        let service = GameSessionService::new(EmptyRepo);

        let error = service
            .submit_result(
                "100001",
                SubmitSessionResultCommand {
                    session_id: "session-1".into(),
                    source_type: "browser_client".into(),
                    source_id: Some("client-1".into()),
                    idempotency_key: "idem-result-1".into(),
                    payload_hash: "hash-1".into(),
                    result_payload: json!({"winner": "user-1"}),
                },
            )
            .await
            .expect_err("unsupported source type must fail");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn create_session_accepts_participant_snapshot() {
        let service = GameSessionService::new(EmptyRepo);

        let item = service
            .create_session("100001", create_command())
            .await
            .expect("session");

        assert_eq!("created", item.status);
        assert_eq!("game-xiangqi", item.game_id);
    }
}
