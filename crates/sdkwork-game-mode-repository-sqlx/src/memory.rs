use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_mode_service::{
    CreateGameModeCommand, GameModeError, GameModeItem, GameModePage, GameModeQuery,
    GameModeRepository, GameModeResult, UpdateGameModeCommand,
};
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone, Default)]
pub struct InMemoryGameModeRepository {
    items: Arc<RwLock<Vec<StoredGameMode>>>,
}

impl InMemoryGameModeRepository {
    pub fn with_seed(items: Vec<(String, GameModeItem)>) -> Self {
        Self {
            items: Arc::new(RwLock::new(
                items
                    .into_iter()
                    .map(|(tenant_id, item)| StoredGameMode { tenant_id, item })
                    .collect(),
            )),
        }
    }
}

#[derive(Clone)]
struct StoredGameMode {
    tenant_id: String,
    item: GameModeItem,
}

#[async_trait]
impl GameModeRepository for InMemoryGameModeRepository {
    async fn list_modes(
        &self,
        tenant_id: &str,
        query: &GameModeQuery,
    ) -> GameModeResult<GameModePage> {
        let items = self.items.read().map_err(lock_error)?;
        let mut filtered: Vec<GameModeItem> = items
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id)
            .filter(|stored| matches_query(&stored.item, query))
            .map(|stored| stored.item.clone())
            .collect();
        filtered.sort_by(|left, right| left.mode_code.cmp(&right.mode_code));

        let total = filtered.len() as u64;
        let offset = query.offset() as usize;
        let limit = query.limit() as usize;
        let page_items = filtered.into_iter().skip(offset).take(limit).collect();

        Ok(GameModePage {
            items: page_items,
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }

    async fn get_mode(&self, tenant_id: &str, mode_id: &str) -> GameModeResult<GameModeItem> {
        let items = self.items.read().map_err(lock_error)?;
        items
            .iter()
            .find(|stored| {
                stored.tenant_id == tenant_id
                    && (stored.item.id == mode_id || stored.item.mode_code == mode_id)
            })
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameModeError::not_found("mode not found"))
    }

    async fn create_mode(
        &self,
        tenant_id: &str,
        command: &CreateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        let mut items = self.items.write().map_err(lock_error)?;
        if items.iter().any(|stored| {
            stored.tenant_id == tenant_id
                && stored.item.game_id == command.game_id
                && stored.item.mode_code == command.mode_code
        }) {
            return Err(GameModeError::invalid("mode_code already exists for game"));
        }

        let item = GameModeItem {
            id: uuid(),
            game_id: command.game_id.clone(),
            mode_code: command.mode_code.clone(),
            title: command.title.clone(),
            status: command.status.clone(),
            min_players: command.min_players,
            max_players: command.max_players,
            team_size: command.team_size,
            ruleset_id: command.ruleset_id.clone(),
            matchmaking_enabled: command.matchmaking_enabled,
            room_enabled: command.room_enabled,
            leaderboard_enabled: command.leaderboard_enabled,
        };
        items.push(StoredGameMode {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }

    async fn update_mode(
        &self,
        tenant_id: &str,
        mode_id: &str,
        command: &UpdateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        let mut items = self.items.write().map_err(lock_error)?;
        let stored = items
            .iter_mut()
            .find(|stored| stored.tenant_id == tenant_id && stored.item.id == mode_id)
            .ok_or_else(|| GameModeError::not_found("mode not found"))?;
        apply_update(&mut stored.item, command);
        Ok(stored.item.clone())
    }
}

fn matches_query(item: &GameModeItem, query: &GameModeQuery) -> bool {
    if let Some(game_id) = query
        .game_id
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
    {
        if item.game_id != game_id {
            return false;
        }
    }
    if let Some(status) = query
        .status
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
    {
        if item.status != status {
            return false;
        }
    }
    if let Some(q) = query.q.as_deref().filter(|value| !is_blank(Some(value))) {
        let needle = q.trim().to_lowercase();
        return item.title.to_lowercase().contains(&needle)
            || item.mode_code.to_lowercase().contains(&needle);
    }
    true
}

fn apply_update(item: &mut GameModeItem, command: &UpdateGameModeCommand) {
    if let Some(title) = &command.title {
        item.title = title.clone();
    }
    if let Some(status) = &command.status {
        item.status = status.clone();
    }
    if let Some(min_players) = command.min_players {
        item.min_players = min_players;
    }
    if let Some(max_players) = command.max_players {
        item.max_players = max_players;
    }
    if let Some(team_size) = command.team_size {
        item.team_size = team_size;
    }
    if let Some(ruleset_id) = &command.ruleset_id {
        item.ruleset_id = ruleset_id.clone();
    }
    if let Some(matchmaking_enabled) = command.matchmaking_enabled {
        item.matchmaking_enabled = matchmaking_enabled;
    }
    if let Some(room_enabled) = command.room_enabled {
        item.room_enabled = room_enabled;
    }
    if let Some(leaderboard_enabled) = command.leaderboard_enabled {
        item.leaderboard_enabled = leaderboard_enabled;
    }
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameModeError {
    GameModeError::invalid("mode repository lock is poisoned")
}
