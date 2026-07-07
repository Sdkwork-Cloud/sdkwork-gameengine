//! SDKWork game ruleset service contracts.

pub mod domain;
pub mod ports;
pub mod service;

pub use domain::models::{
    CreateGameRulesetCommand, GameRulesetError, GameRulesetItem, GameRulesetResult,
};
pub use ports::repository::GameRulesetRepository;
pub use service::GameRulesetService;

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;

    struct EmptyRepo;

    #[async_trait]
    impl GameRulesetRepository for EmptyRepo {
        async fn get_active_ruleset(
            &self,
            _tenant_id: &str,
            _game_id: &str,
            _mode_id: Option<&str>,
        ) -> GameRulesetResult<GameRulesetItem> {
            Ok(GameRulesetItem {
                id: "ruleset-1".into(),
                game_id: "game-1".into(),
                mode_id: Some("mode-1".into()),
                ruleset_code: "ranked-default".into(),
                version_no: 1,
                status: "active".into(),
                config_schema: serde_json::json!({}),
                config_values: serde_json::json!({ "score": "win_loss" }),
                activated_at: Some("2026-01-01T00:00:00Z".into()),
            })
        }

        async fn create_ruleset(
            &self,
            _tenant_id: &str,
            command: &CreateGameRulesetCommand,
        ) -> GameRulesetResult<GameRulesetItem> {
            Ok(GameRulesetItem {
                id: "ruleset-1".into(),
                game_id: command.game_id.clone(),
                mode_id: command.mode_id.clone(),
                ruleset_code: command.ruleset_code.clone(),
                version_no: command.version_no,
                status: command.status.clone(),
                config_schema: command.config_schema.clone(),
                config_values: command.config_values.clone(),
                activated_at: None,
            })
        }
    }

    #[tokio::test]
    async fn create_ruleset_rejects_invalid_version() {
        let service = GameRulesetService::new(EmptyRepo);

        let result = service
            .create_ruleset(
                "100001",
                CreateGameRulesetCommand {
                    game_id: "game-1".into(),
                    mode_id: None,
                    ruleset_code: "ranked-default".into(),
                    version_no: 0,
                    status: "draft".into(),
                    config_schema: serde_json::json!({}),
                    config_values: serde_json::json!({}),
                },
            )
            .await;

        assert_eq!(result.unwrap_err().code(), "invalid");
    }

    #[tokio::test]
    async fn active_ruleset_requires_game_id() {
        let service = GameRulesetService::new(EmptyRepo);

        let result = service.get_active_ruleset("100001", "", None).await;

        assert_eq!(result.unwrap_err().code(), "invalid");
    }

    #[tokio::test]
    async fn active_ruleset_returns_repository_item() {
        let service = GameRulesetService::new(EmptyRepo);

        let item = service
            .get_active_ruleset("100001", "game-1", Some("mode-1"))
            .await
            .expect("ruleset");

        assert_eq!(item.status, "active");
    }
}
