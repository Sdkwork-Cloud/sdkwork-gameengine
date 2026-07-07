use async_trait::async_trait;

use crate::domain::models::{
    CreateGameSessionCommand, GameSessionItem, GameSessionParticipantItem, GameSessionResult,
    GameSessionResultItem, StartGameSessionCommand, SubmitSessionResultCommand,
};

#[async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn create_session(
        &self,
        tenant_id: &str,
        command: &CreateGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem>;

    async fn get_session(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<GameSessionItem>;

    async fn list_participants(
        &self,
        tenant_id: &str,
        session_id: &str,
    ) -> GameSessionResult<Vec<GameSessionParticipantItem>>;

    async fn start_session(
        &self,
        tenant_id: &str,
        command: &StartGameSessionCommand,
    ) -> GameSessionResult<GameSessionItem>;

    async fn submit_result(
        &self,
        tenant_id: &str,
        command: &SubmitSessionResultCommand,
    ) -> GameSessionResult<GameSessionResultItem>;
}
