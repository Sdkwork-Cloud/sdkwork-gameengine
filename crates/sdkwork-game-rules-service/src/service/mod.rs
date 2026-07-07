use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    CreateGameRulesetCommand, GameRulesetError, GameRulesetItem, GameRulesetResult,
};
use crate::ports::repository::GameRulesetRepository;

pub struct GameRulesetService<R> {
    repository: R,
}

impl<R> GameRulesetService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameRulesetService<R>
where
    R: GameRulesetRepository,
{
    pub async fn get_active_ruleset(
        &self,
        tenant_id: &str,
        game_id: &str,
        mode_id: Option<&str>,
    ) -> GameRulesetResult<GameRulesetItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("game_id", game_id)?;
        if let Some(mode_id) = mode_id {
            validate_required("mode_id", mode_id)?;
        }
        self.repository
            .get_active_ruleset(tenant_id, game_id, mode_id)
            .await
    }

    pub async fn create_ruleset(
        &self,
        tenant_id: &str,
        command: CreateGameRulesetCommand,
    ) -> GameRulesetResult<GameRulesetItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("game_id", &command.game_id)?;
        validate_required("ruleset_code", &command.ruleset_code)?;
        validate_version(command.version_no)?;
        validate_ruleset_status(&command.status)?;
        self.repository.create_ruleset(tenant_id, &command).await
    }
}

fn validate_required(field: &str, value: &str) -> GameRulesetResult<()> {
    if is_blank(Some(value)) {
        return Err(GameRulesetError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_version(version_no: i32) -> GameRulesetResult<()> {
    if version_no < 1 {
        return Err(GameRulesetError::invalid("version_no must be positive"));
    }
    Ok(())
}

fn validate_ruleset_status(status: &str) -> GameRulesetResult<()> {
    if matches!(status, "draft" | "active" | "deprecated" | "archived") {
        return Ok(());
    }
    Err(GameRulesetError::invalid("ruleset status is not supported"))
}
