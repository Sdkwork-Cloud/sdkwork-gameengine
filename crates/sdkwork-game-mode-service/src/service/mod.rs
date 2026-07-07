use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    CreateGameModeCommand, GameModeError, GameModeItem, GameModePage, GameModeQuery,
    GameModeResult, UpdateGameModeCommand,
};
use crate::ports::repository::GameModeRepository;

pub struct GameModeService<R> {
    repository: R,
}

impl<R> GameModeService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameModeService<R>
where
    R: GameModeRepository,
{
    pub async fn list_modes(
        &self,
        tenant_id: &str,
        query: GameModeQuery,
    ) -> GameModeResult<GameModePage> {
        validate_required("tenant_id", tenant_id)?;
        self.repository.list_modes(tenant_id, &query).await
    }

    pub async fn get_mode(&self, tenant_id: &str, mode_id: &str) -> GameModeResult<GameModeItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("mode_id", mode_id)?;
        self.repository.get_mode(tenant_id, mode_id).await
    }

    pub async fn create_mode(
        &self,
        tenant_id: &str,
        command: CreateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("game_id", &command.game_id)?;
        validate_required("mode_code", &command.mode_code)?;
        validate_required("title", &command.title)?;
        validate_mode_status(&command.status)?;
        validate_player_range(command.min_players, command.max_players)?;
        validate_team_size(command.team_size, Some(command.max_players))?;
        self.repository.create_mode(tenant_id, &command).await
    }

    pub async fn update_mode(
        &self,
        tenant_id: &str,
        mode_id: &str,
        command: UpdateGameModeCommand,
    ) -> GameModeResult<GameModeItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("mode_id", mode_id)?;
        if let Some(title) = command.title.as_deref() {
            validate_required("title", title)?;
        }
        if let Some(status) = command.status.as_deref() {
            validate_mode_status(status)?;
        }
        if let Some(min_players) = command.min_players {
            validate_min_players(min_players)?;
        }
        if let Some(max_players) = command.max_players {
            validate_min_players(max_players)?;
        }
        if let (Some(min_players), Some(max_players)) = (command.min_players, command.max_players) {
            validate_player_range(min_players, max_players)?;
        }
        if let Some(team_size) = command.team_size {
            validate_team_size(team_size, command.max_players)?;
        }
        self.repository
            .update_mode(tenant_id, mode_id, &command)
            .await
    }
}

fn validate_required(field: &str, value: &str) -> GameModeResult<()> {
    if is_blank(Some(value)) {
        return Err(GameModeError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_mode_status(status: &str) -> GameModeResult<()> {
    if matches!(status, "draft" | "active" | "disabled" | "archived") {
        return Ok(());
    }
    Err(GameModeError::invalid("mode status is not supported"))
}

fn validate_min_players(value: i32) -> GameModeResult<()> {
    if value < 1 {
        return Err(GameModeError::invalid("player count must be positive"));
    }
    Ok(())
}

fn validate_player_range(min_players: i32, max_players: i32) -> GameModeResult<()> {
    validate_min_players(min_players)?;
    validate_min_players(max_players)?;
    if max_players < min_players {
        return Err(GameModeError::invalid(
            "max_players must be greater than or equal to min_players",
        ));
    }
    Ok(())
}

fn validate_team_size(team_size: Option<i32>, max_players: Option<i32>) -> GameModeResult<()> {
    let Some(team_size) = team_size else {
        return Ok(());
    };
    validate_min_players(team_size)?;
    if let Some(max_players) = max_players {
        if team_size > max_players {
            return Err(GameModeError::invalid(
                "team_size must be less than or equal to max_players",
            ));
        }
    }
    Ok(())
}
