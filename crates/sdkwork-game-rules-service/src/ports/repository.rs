use async_trait::async_trait;

use crate::domain::models::{CreateGameRulesetCommand, GameRulesetItem, GameRulesetResult};

#[async_trait]
pub trait GameRulesetRepository: Send + Sync {
    async fn get_active_ruleset(
        &self,
        tenant_id: &str,
        game_id: &str,
        mode_id: Option<&str>,
    ) -> GameRulesetResult<GameRulesetItem>;

    async fn create_ruleset(
        &self,
        tenant_id: &str,
        command: &CreateGameRulesetCommand,
    ) -> GameRulesetResult<GameRulesetItem>;
}
