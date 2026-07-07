use async_trait::async_trait;

use crate::domain::models::{
    CreateGameModeCommand, GameModeItem, GameModePage, GameModeQuery, GameModeResult,
    UpdateGameModeCommand,
};

#[async_trait]
pub trait GameModeRepository: Send + Sync {
    async fn list_modes(
        &self,
        tenant_id: &str,
        query: &GameModeQuery,
    ) -> GameModeResult<GameModePage>;

    async fn get_mode(&self, tenant_id: &str, mode_id: &str) -> GameModeResult<GameModeItem>;

    async fn create_mode(
        &self,
        tenant_id: &str,
        command: &CreateGameModeCommand,
    ) -> GameModeResult<GameModeItem>;

    async fn update_mode(
        &self,
        tenant_id: &str,
        mode_id: &str,
        command: &UpdateGameModeCommand,
    ) -> GameModeResult<GameModeItem>;
}
