//! SQLx-backed SDKWork game mode repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameModeRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameModeRepository;
pub use sqlx::SqlxGameModeRepository;

#[cfg(test)]
mod tests {
    use sdkwork_game_mode_service::{
        CreateGameModeCommand, GameModeItem, GameModeQuery, GameModeRepository,
        UpdateGameModeCommand,
    };

    use super::*;

    fn mode_item(id: &str, tenant_id: &str, status: &str) -> (String, GameModeItem) {
        (
            tenant_id.into(),
            GameModeItem {
                id: id.into(),
                game_id: "game-1".into(),
                mode_code: id.into(),
                title: id.into(),
                status: status.into(),
                min_players: 2,
                max_players: 4,
                team_size: Some(2),
                ruleset_id: None,
                matchmaking_enabled: true,
                room_enabled: true,
                leaderboard_enabled: true,
            },
        )
    }

    #[tokio::test]
    async fn memory_repository_lists_modes_by_tenant_and_status() {
        let repository = InMemoryGameModeRepository::with_seed(vec![
            mode_item("ranked", "100001", "active"),
            mode_item("draft", "100001", "draft"),
            mode_item("other", "200002", "active"),
        ]);

        let page = repository
            .list_modes(
                "100001",
                &GameModeQuery {
                    status: Some("active".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("mode page");

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].mode_code, "ranked");
    }

    #[tokio::test]
    async fn memory_repository_creates_and_updates_mode() {
        let repository = InMemoryGameModeRepository::default();

        let created = repository
            .create_mode(
                "100001",
                &CreateGameModeCommand {
                    game_id: "game-1".into(),
                    mode_code: "ranked".into(),
                    title: "Ranked".into(),
                    status: "active".into(),
                    min_players: 2,
                    max_players: 4,
                    team_size: Some(2),
                    ruleset_id: None,
                    matchmaking_enabled: true,
                    room_enabled: true,
                    leaderboard_enabled: true,
                },
            )
            .await
            .expect("created mode");

        let updated = repository
            .update_mode(
                "100001",
                &created.id,
                &UpdateGameModeCommand {
                    title: Some("Ranked v2".into()),
                    max_players: Some(6),
                    ..Default::default()
                },
            )
            .await
            .expect("updated mode");

        assert_eq!(updated.title, "Ranked v2");
        assert_eq!(updated.max_players, 6);
    }
}
