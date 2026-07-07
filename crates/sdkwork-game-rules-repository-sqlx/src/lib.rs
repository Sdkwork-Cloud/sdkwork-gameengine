//! SQLx-backed SDKWork game ruleset repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameRulesetRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameRulesetRepository;
pub use sqlx::SqlxGameRulesetRepository;

#[cfg(test)]
mod tests {
    use sdkwork_game_rules_service::{
        CreateGameRulesetCommand, GameRulesetItem, GameRulesetRepository,
    };

    use super::*;

    fn ruleset_item(
        id: &str,
        tenant_id: &str,
        mode_id: Option<&str>,
        status: &str,
    ) -> (String, GameRulesetItem) {
        (
            tenant_id.into(),
            GameRulesetItem {
                id: id.into(),
                game_id: "game-1".into(),
                mode_id: mode_id.map(String::from),
                ruleset_code: id.into(),
                version_no: 1,
                status: status.into(),
                config_schema: serde_json::json!({}),
                config_values: serde_json::json!({ "score": "win_loss" }),
                activated_at: Some("2026-01-01T00:00:00Z".into()),
            },
        )
    }

    #[tokio::test]
    async fn memory_repository_returns_active_ruleset_by_tenant_game_and_mode() {
        let repository = InMemoryGameRulesetRepository::with_seed(vec![
            ruleset_item("draft-rules", "100001", Some("mode-1"), "draft"),
            ruleset_item("active-rules", "100001", Some("mode-1"), "active"),
            ruleset_item("other-tenant", "200002", Some("mode-1"), "active"),
        ]);

        let item = repository
            .get_active_ruleset("100001", "game-1", Some("mode-1"))
            .await
            .expect("active ruleset");

        assert_eq!(item.ruleset_code, "active-rules");
    }

    #[tokio::test]
    async fn memory_repository_creates_ruleset() {
        let repository = InMemoryGameRulesetRepository::default();

        let item = repository
            .create_ruleset(
                "100001",
                &CreateGameRulesetCommand {
                    game_id: "game-1".into(),
                    mode_id: None,
                    ruleset_code: "default".into(),
                    version_no: 1,
                    status: "draft".into(),
                    config_schema: serde_json::json!({}),
                    config_values: serde_json::json!({ "score": "win_loss" }),
                },
            )
            .await
            .expect("created ruleset");

        assert_eq!(item.ruleset_code, "default");
    }
}
