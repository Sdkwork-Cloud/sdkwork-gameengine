use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_rules_service::{
    CreateGameRulesetCommand, GameRulesetError, GameRulesetItem, GameRulesetRepository,
    GameRulesetResult,
};
use sdkwork_utils_rust::id::uuid;

#[derive(Clone, Default)]
pub struct InMemoryGameRulesetRepository {
    items: Arc<RwLock<Vec<StoredRuleset>>>,
}

impl InMemoryGameRulesetRepository {
    pub fn with_seed(items: Vec<(String, GameRulesetItem)>) -> Self {
        Self {
            items: Arc::new(RwLock::new(
                items
                    .into_iter()
                    .map(|(tenant_id, item)| StoredRuleset { tenant_id, item })
                    .collect(),
            )),
        }
    }
}

#[derive(Clone)]
struct StoredRuleset {
    tenant_id: String,
    item: GameRulesetItem,
}

#[async_trait]
impl GameRulesetRepository for InMemoryGameRulesetRepository {
    async fn get_active_ruleset(
        &self,
        tenant_id: &str,
        game_id: &str,
        mode_id: Option<&str>,
    ) -> GameRulesetResult<GameRulesetItem> {
        let items = self.items.read().map_err(lock_error)?;
        items
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id)
            .filter(|stored| stored.item.game_id == game_id)
            .filter(|stored| stored.item.status == "active")
            .filter(|stored| stored.item.mode_id.as_deref() == mode_id)
            .max_by_key(|stored| stored.item.version_no)
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameRulesetError::not_found("active ruleset not found"))
    }

    async fn create_ruleset(
        &self,
        tenant_id: &str,
        command: &CreateGameRulesetCommand,
    ) -> GameRulesetResult<GameRulesetItem> {
        let mut items = self.items.write().map_err(lock_error)?;
        let item = GameRulesetItem {
            id: uuid(),
            game_id: command.game_id.clone(),
            mode_id: command.mode_id.clone(),
            ruleset_code: command.ruleset_code.clone(),
            version_no: command.version_no,
            status: command.status.clone(),
            config_schema: command.config_schema.clone(),
            config_values: command.config_values.clone(),
            activated_at: None,
        };
        items.push(StoredRuleset {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameRulesetError {
    GameRulesetError::invalid("ruleset repository lock is poisoned")
}
