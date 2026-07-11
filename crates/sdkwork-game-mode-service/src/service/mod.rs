use sdkwork_utils_rust::string::is_blank;
use sdkwork_utils_rust::validated_offset_list_params;

use crate::domain::models::{
    CreateGameModeCommand, GameModeError, GameModeItem, GameModePage, GameModeQuery,
    GameModeResult, UpdateGameModeCommand,
};
use crate::ports::repository::GameModeRepository;

pub const MAX_MODE_PLAYERS: i32 = 64;

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
        validate_pagination(query.page, query.page_size)?;
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
            validate_max_players(max_players)?;
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

fn validate_pagination(page: Option<u32>, page_size: Option<u32>) -> GameModeResult<()> {
    validated_offset_list_params(page.map(i64::from), page_size.map(i64::from))
        .map(|_| ())
        .map_err(|_| {
            GameModeError::invalid_parameter(
                "page and page_size must follow SDKWork pagination bounds",
            )
        })
}

fn validate_mode_status(status: &str) -> GameModeResult<()> {
    if matches!(status, "draft" | "active" | "disabled" | "archived") {
        return Ok(());
    }
    Err(GameModeError::invalid("mode status is not supported"))
}

fn validate_min_players(value: i32) -> GameModeResult<()> {
    if !(1..=MAX_MODE_PLAYERS).contains(&value) {
        return Err(GameModeError::invalid(format!(
            "min_players must be between 1 and {MAX_MODE_PLAYERS}"
        )));
    }
    Ok(())
}

fn validate_max_players(value: i32) -> GameModeResult<()> {
    if !(1..=MAX_MODE_PLAYERS).contains(&value) {
        return Err(GameModeError::invalid(format!(
            "max_players must be between 1 and {MAX_MODE_PLAYERS}"
        )));
    }
    Ok(())
}

fn validate_player_range(min_players: i32, max_players: i32) -> GameModeResult<()> {
    validate_min_players(min_players)?;
    validate_max_players(max_players)?;
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
    if !(1..=MAX_MODE_PLAYERS).contains(&team_size) {
        return Err(GameModeError::invalid(format!(
            "team_size must be between 1 and {MAX_MODE_PLAYERS}"
        )));
    }
    if let Some(max_players) = max_players {
        if team_size > max_players {
            return Err(GameModeError::invalid(
                "team_size must be less than or equal to max_players",
            ));
        }
    }
    Ok(())
}
