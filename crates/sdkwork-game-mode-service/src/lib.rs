//! SDKWork game mode service contracts.

pub mod domain;
pub mod ports;
pub mod service;

pub use domain::models::{
    CreateGameModeCommand, GameModeError, GameModeItem, GameModePage, GameModeQuery,
    GameModeResult, UpdateGameModeCommand,
};
pub use ports::repository::GameModeRepository;
pub use service::GameModeService;

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;

    struct EmptyRepo;

    #[async_trait]
    impl GameModeRepository for EmptyRepo {
        async fn list_modes(
            &self,
            _tenant_id: &str,
            query: &GameModeQuery,
        ) -> GameModeResult<GameModePage> {
            Ok(GameModePage {
                items: vec![],
                total: 0,
                page: query.page.unwrap_or(1),
                page_size: query.limit(),
            })
        }

        async fn get_mode(&self, _tenant_id: &str, _mode_id: &str) -> GameModeResult<GameModeItem> {
            Err(GameModeError::not_found("mode not found"))
        }

        async fn create_mode(
            &self,
            _tenant_id: &str,
            command: &CreateGameModeCommand,
        ) -> GameModeResult<GameModeItem> {
            Ok(GameModeItem {
                id: "mode-1".into(),
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
            })
        }

        async fn update_mode(
            &self,
            _tenant_id: &str,
            _mode_id: &str,
            command: &UpdateGameModeCommand,
        ) -> GameModeResult<GameModeItem> {
            Ok(GameModeItem {
                id: "mode-1".into(),
                game_id: "game-1".into(),
                mode_code: "ranked".into(),
                title: command.title.clone().unwrap_or_else(|| "Ranked".into()),
                status: command.status.clone().unwrap_or_else(|| "active".into()),
                min_players: command.min_players.unwrap_or(2),
                max_players: command.max_players.unwrap_or(2),
                team_size: command.team_size.unwrap_or(None),
                ruleset_id: command.ruleset_id.clone().unwrap_or(None),
                matchmaking_enabled: command.matchmaking_enabled.unwrap_or(true),
                room_enabled: command.room_enabled.unwrap_or(true),
                leaderboard_enabled: command.leaderboard_enabled.unwrap_or(true),
            })
        }
    }

    #[test]
    fn mode_query_clamps_page_size() {
        let query = GameModeQuery {
            page: Some(2),
            page_size: Some(500),
            ..Default::default()
        };

        assert_eq!(query.limit(), 200);
        assert_eq!(query.offset(), 200);
    }

    #[tokio::test]
    async fn create_mode_rejects_invalid_player_range() {
        let service = GameModeService::new(EmptyRepo);

        let result = service
            .create_mode(
                "100001",
                CreateGameModeCommand {
                    game_id: "game-1".into(),
                    mode_code: "ranked".into(),
                    title: "Ranked".into(),
                    status: "active".into(),
                    min_players: 4,
                    max_players: 2,
                    team_size: None,
                    ruleset_id: None,
                    matchmaking_enabled: true,
                    room_enabled: true,
                    leaderboard_enabled: true,
                },
            )
            .await;

        assert_eq!(result.unwrap_err().code(), "invalid");
    }

    #[tokio::test]
    async fn create_mode_accepts_active_mode() {
        let service = GameModeService::new(EmptyRepo);

        let item = service
            .create_mode(
                "100001",
                CreateGameModeCommand {
                    game_id: "game-1".into(),
                    mode_code: "ranked".into(),
                    title: "Ranked".into(),
                    status: "active".into(),
                    min_players: 2,
                    max_players: 4,
                    team_size: Some(2),
                    ruleset_id: Some("ruleset-1".into()),
                    matchmaking_enabled: true,
                    room_enabled: true,
                    leaderboard_enabled: true,
                },
            )
            .await
            .expect("mode");

        assert_eq!(item.mode_code, "ranked");
    }
}
