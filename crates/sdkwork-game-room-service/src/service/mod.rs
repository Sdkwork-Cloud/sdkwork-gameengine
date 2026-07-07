use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomError, GameRoomItem, GameRoomPage,
    GameRoomQuery, GameRoomResult, GameRoomSeatItem, JoinGameRoomCommand, LeaveGameRoomCommand,
    ReadyGameRoomCommand, StartGameRoomCommand,
};
use crate::ports::repository::GameRoomRepository;

pub const MAX_ROOM_PLAYERS: i32 = 64;

pub struct GameRoomService<R> {
    repository: R,
}

impl<R> GameRoomService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameRoomService<R>
where
    R: GameRoomRepository,
{
    pub async fn list_rooms(
        &self,
        tenant_id: &str,
        query: GameRoomQuery,
    ) -> GameRoomResult<GameRoomPage> {
        validate_required("tenant_id", tenant_id)?;
        self.repository.list_rooms(tenant_id, &query).await
    }

    pub async fn get_room(&self, tenant_id: &str, room_id: &str) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", room_id)?;
        self.repository.get_room(tenant_id, room_id).await
    }

    pub async fn list_room_seats(
        &self,
        tenant_id: &str,
        room_id: &str,
    ) -> GameRoomResult<Vec<GameRoomSeatItem>> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", room_id)?;
        self.repository.list_room_seats(tenant_id, room_id).await
    }

    pub async fn create_room(
        &self,
        tenant_id: &str,
        command: CreateGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("game_id", &command.game_id)?;
        validate_required("room_code", &command.room_code)?;
        validate_required("host_user_id", &command.host_user_id)?;
        validate_visibility(&command.visibility)?;
        validate_join_policy(&command.join_policy)?;
        validate_max_players(command.max_players)?;
        self.repository.create_room(tenant_id, &command).await
    }

    pub async fn join_room(
        &self,
        tenant_id: &str,
        command: JoinGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", &command.room_id)?;
        validate_required("user_id", &command.user_id)?;
        let room = self
            .repository
            .get_room(tenant_id, &command.room_id)
            .await?;
        validate_expected_version(&room, command.expected_version)?;
        if room.status != "open" {
            return Err(GameRoomError::invalid("room is not open"));
        }
        if room.join_policy != "open" {
            return Err(GameRoomError::invalid("room is not open for direct join"));
        }
        let seats = self
            .repository
            .list_room_seats(tenant_id, &command.room_id)
            .await?;
        if active_seat_for_user(&seats, &command.user_id).is_some() {
            return Err(GameRoomError::invalid("user already joined room"));
        }
        if active_seat_count(&seats) >= room.max_players {
            return Err(GameRoomError::invalid("room is full"));
        }
        self.repository.join_room(tenant_id, &command).await
    }

    pub async fn leave_room(
        &self,
        tenant_id: &str,
        command: LeaveGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", &command.room_id)?;
        validate_required("user_id", &command.user_id)?;
        let room = self
            .repository
            .get_room(tenant_id, &command.room_id)
            .await?;
        validate_expected_version(&room, command.expected_version)?;
        if room.status == "closed" {
            return Err(GameRoomError::invalid("room is closed"));
        }
        let seats = self
            .repository
            .list_room_seats(tenant_id, &command.room_id)
            .await?;
        active_seat_for_user(&seats, &command.user_id)
            .ok_or_else(|| GameRoomError::not_found("active room seat not found"))?;
        self.repository.leave_room(tenant_id, &command).await
    }

    pub async fn set_ready(
        &self,
        tenant_id: &str,
        command: ReadyGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", &command.room_id)?;
        validate_required("user_id", &command.user_id)?;
        let room = self
            .repository
            .get_room(tenant_id, &command.room_id)
            .await?;
        validate_expected_version(&room, command.expected_version)?;
        if room.status != "open" {
            return Err(GameRoomError::invalid("room is not open"));
        }
        let seats = self
            .repository
            .list_room_seats(tenant_id, &command.room_id)
            .await?;
        active_seat_for_user(&seats, &command.user_id)
            .ok_or_else(|| GameRoomError::not_found("active room seat not found"))?;
        self.repository.set_ready(tenant_id, &command).await
    }

    pub async fn start_room(
        &self,
        tenant_id: &str,
        command: StartGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", &command.room_id)?;
        validate_required("host_user_id", &command.host_user_id)?;
        let room = self
            .repository
            .get_room(tenant_id, &command.room_id)
            .await?;
        validate_expected_version(&room, command.expected_version)?;
        validate_host(&room, &command.host_user_id)?;
        if room.status != "open" {
            return Err(GameRoomError::invalid("room is not open"));
        }
        let seats = self
            .repository
            .list_room_seats(tenant_id, &command.room_id)
            .await?;
        let active_seats: Vec<&GameRoomSeatItem> =
            seats.iter().filter(|seat| is_active_seat(seat)).collect();
        if active_seats.is_empty() {
            return Err(GameRoomError::invalid("room has no active players"));
        }
        if active_seats.iter().any(|seat| seat.status != "ready") {
            return Err(GameRoomError::invalid("all active players must be ready"));
        }
        self.repository.start_room(tenant_id, &command).await
    }

    pub async fn close_room(
        &self,
        tenant_id: &str,
        command: CloseGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", &command.room_id)?;
        validate_required("operator_user_id", &command.operator_user_id)?;
        let room = self
            .repository
            .get_room(tenant_id, &command.room_id)
            .await?;
        validate_expected_version(&room, command.expected_version)?;
        validate_host(&room, &command.operator_user_id)?;
        if room.status == "closed" {
            return Err(GameRoomError::invalid("room is already closed"));
        }
        self.repository.close_room(tenant_id, &command).await
    }

    pub async fn force_close_room(
        &self,
        tenant_id: &str,
        command: CloseGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("room_id", &command.room_id)?;
        validate_required("operator_user_id", &command.operator_user_id)?;
        let room = self
            .repository
            .get_room(tenant_id, &command.room_id)
            .await?;
        validate_expected_version(&room, command.expected_version)?;
        if room.status == "closed" {
            return Err(GameRoomError::invalid("room is already closed"));
        }
        self.repository.close_room(tenant_id, &command).await
    }
}

fn validate_required(field: &str, value: &str) -> GameRoomResult<()> {
    if is_blank(Some(value)) {
        return Err(GameRoomError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_visibility(value: &str) -> GameRoomResult<()> {
    if matches!(value, "public" | "private") {
        return Ok(());
    }
    Err(GameRoomError::invalid("room visibility is not supported"))
}

fn validate_join_policy(value: &str) -> GameRoomResult<()> {
    if matches!(value, "open" | "invite" | "password") {
        return Ok(());
    }
    Err(GameRoomError::invalid("room join_policy is not supported"))
}

fn validate_max_players(value: i32) -> GameRoomResult<()> {
    if !(1..=MAX_ROOM_PLAYERS).contains(&value) {
        return Err(GameRoomError::invalid(format!(
            "max_players must be between 1 and {MAX_ROOM_PLAYERS}"
        )));
    }
    Ok(())
}

fn validate_expected_version(room: &GameRoomItem, expected: Option<i64>) -> GameRoomResult<()> {
    if let Some(expected) = expected {
        if room.version != expected {
            return Err(GameRoomError::conflict("room version has changed"));
        }
    }
    Ok(())
}

fn validate_host(room: &GameRoomItem, user_id: &str) -> GameRoomResult<()> {
    if room.host_user_id != user_id {
        return Err(GameRoomError::forbidden(
            "only room host can perform this action",
        ));
    }
    Ok(())
}

fn active_seat_count(seats: &[GameRoomSeatItem]) -> i32 {
    seats.iter().filter(|seat| is_active_seat(seat)).count() as i32
}

fn active_seat_for_user<'a>(
    seats: &'a [GameRoomSeatItem],
    user_id: &str,
) -> Option<&'a GameRoomSeatItem> {
    seats.iter().find(|seat| {
        is_active_seat(seat)
            && seat
                .user_id
                .as_deref()
                .is_some_and(|value| value == user_id)
    })
}

fn is_active_seat(seat: &GameRoomSeatItem) -> bool {
    matches!(
        seat.status.as_str(),
        "reserved" | "joined" | "ready" | "playing"
    )
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::domain::models::GameRoomPage;

    struct EmptyRepo;

    #[async_trait]
    impl GameRoomRepository for EmptyRepo {
        async fn list_rooms(
            &self,
            _tenant_id: &str,
            _query: &GameRoomQuery,
        ) -> GameRoomResult<GameRoomPage> {
            Ok(GameRoomPage {
                items: vec![],
                total: 0,
                page: 1,
                page_size: 20,
            })
        }

        async fn get_room(&self, _tenant_id: &str, _room_id: &str) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::not_found("room not found"))
        }

        async fn list_room_seats(
            &self,
            _tenant_id: &str,
            _room_id: &str,
        ) -> GameRoomResult<Vec<GameRoomSeatItem>> {
            Ok(vec![])
        }

        async fn create_room(
            &self,
            _tenant_id: &str,
            _command: &CreateGameRoomCommand,
        ) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::invalid("not implemented"))
        }

        async fn join_room(
            &self,
            _tenant_id: &str,
            _command: &JoinGameRoomCommand,
        ) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::invalid("not implemented"))
        }

        async fn leave_room(
            &self,
            _tenant_id: &str,
            _command: &LeaveGameRoomCommand,
        ) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::invalid("not implemented"))
        }

        async fn set_ready(
            &self,
            _tenant_id: &str,
            _command: &ReadyGameRoomCommand,
        ) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::invalid("not implemented"))
        }

        async fn start_room(
            &self,
            _tenant_id: &str,
            _command: &StartGameRoomCommand,
        ) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::invalid("not implemented"))
        }

        async fn close_room(
            &self,
            _tenant_id: &str,
            _command: &CloseGameRoomCommand,
        ) -> GameRoomResult<GameRoomItem> {
            Err(GameRoomError::invalid("not implemented"))
        }
    }

    #[tokio::test]
    async fn list_rooms_rejects_empty_tenant() {
        let service = GameRoomService::new(EmptyRepo);
        let result = service.list_rooms("", GameRoomQuery::default()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "invalid");
    }

    #[tokio::test]
    async fn create_room_rejects_excessive_max_players_before_repository_access() {
        let service = GameRoomService::new(EmptyRepo);
        let result = service
            .create_room(
                "100001",
                CreateGameRoomCommand {
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    ruleset_id: None,
                    room_code: "ROOM-TOO-LARGE".into(),
                    host_user_id: "user-host".into(),
                    visibility: "public".into(),
                    join_policy: "open".into(),
                    max_players: 65,
                },
            )
            .await;

        let error = result.unwrap_err();
        assert_eq!("invalid", error.code());
        assert_eq!("max_players must be between 1 and 64", error.message());
    }
}
