use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_room_service::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomError, GameRoomItem, GameRoomPage,
    GameRoomQuery, GameRoomRepository, GameRoomResult, GameRoomSeatItem, JoinGameRoomCommand,
    LeaveGameRoomCommand, ReadyGameRoomCommand, StartGameRoomCommand,
};
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone, Default)]
pub struct InMemoryGameRoomRepository {
    store: Arc<RwLock<RoomStore>>,
}

impl InMemoryGameRoomRepository {
    pub fn with_seed(items: Vec<GameRoomItem>) -> Self {
        Self {
            store: Arc::new(RwLock::new(RoomStore {
                rooms: items
                    .into_iter()
                    .map(|item| StoredRoom {
                        tenant_id: "100001".into(),
                        item,
                    })
                    .collect(),
                seats: vec![],
            })),
        }
    }
}

#[derive(Default)]
struct RoomStore {
    rooms: Vec<StoredRoom>,
    seats: Vec<StoredSeat>,
}

#[derive(Clone)]
struct StoredRoom {
    tenant_id: String,
    item: GameRoomItem,
}

#[derive(Clone)]
struct StoredSeat {
    tenant_id: String,
    item: GameRoomSeatItem,
}

fn matches_query(item: &GameRoomItem, query: &GameRoomQuery) -> bool {
    if let Some(game_id) = &query.game_id {
        if item.game_id != *game_id {
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

    true
}

#[async_trait]
impl GameRoomRepository for InMemoryGameRoomRepository {
    async fn list_rooms(
        &self,
        tenant_id: &str,
        query: &GameRoomQuery,
    ) -> GameRoomResult<GameRoomPage> {
        let store = self.store.read().map_err(lock_error)?;
        let mut filtered: Vec<GameRoomItem> = store
            .rooms
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id)
            .filter(|stored| matches_query(&stored.item, query))
            .map(|stored| stored.item.clone())
            .collect();
        filtered.sort_by(|left, right| right.version.cmp(&left.version));

        let total = filtered.len() as u64;
        let offset = query.offset() as usize;
        let limit = query.limit() as usize;
        let page_items = filtered.into_iter().skip(offset).take(limit).collect();

        Ok(GameRoomPage {
            items: page_items,
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }

    async fn get_room(&self, tenant_id: &str, room_id: &str) -> GameRoomResult<GameRoomItem> {
        let store = self.store.read().map_err(lock_error)?;
        store
            .rooms
            .iter()
            .find(|stored| {
                stored.tenant_id == tenant_id
                    && (stored.item.id == room_id || stored.item.room_code == room_id)
            })
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameRoomError::not_found("room not found"))
    }

    async fn list_room_seats(
        &self,
        tenant_id: &str,
        room_id: &str,
    ) -> GameRoomResult<Vec<GameRoomSeatItem>> {
        let store = self.store.read().map_err(lock_error)?;
        let mut seats: Vec<GameRoomSeatItem> = store
            .seats
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id && stored.item.room_id == room_id)
            .map(|stored| stored.item.clone())
            .collect();
        seats.sort_by_key(|seat| seat.seat_no);
        Ok(seats)
    }

    async fn create_room(
        &self,
        tenant_id: &str,
        command: &CreateGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        if store.rooms.iter().any(|stored| {
            stored.tenant_id == tenant_id && stored.item.room_code == command.room_code
        }) {
            return Err(GameRoomError::invalid("room_code already exists"));
        }

        let room_id = uuid();
        let current_players = if is_blank(Some(&command.host_user_id)) {
            0
        } else {
            1
        };
        let item = GameRoomItem {
            id: room_id.clone(),
            game_id: command.game_id.clone(),
            mode_id: command.mode_id.clone(),
            ruleset_id: command.ruleset_id.clone(),
            room_code: command.room_code.clone(),
            host_user_id: command.host_user_id.clone(),
            visibility: command.visibility.clone(),
            join_policy: command.join_policy.clone(),
            max_players: command.max_players,
            current_players,
            status: "open".into(),
            version: 0,
        };

        store.rooms.push(StoredRoom {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });

        if current_players == 1 {
            store.seats.push(StoredSeat {
                tenant_id: tenant_id.into(),
                item: GameRoomSeatItem {
                    id: uuid(),
                    room_id,
                    seat_no: 1,
                    team_no: None,
                    user_id: Some(command.host_user_id.clone()),
                    display_name_snapshot: None,
                    status: "joined".into(),
                    version: 0,
                },
            });
        }

        Ok(item)
    }

    async fn join_room(
        &self,
        tenant_id: &str,
        command: &JoinGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let room_index = find_room_index(&store, tenant_id, &command.room_id)?;
        ensure_expected_version(&store.rooms[room_index].item, command.expected_version)?;
        let seat_no = next_available_seat_no(&store, tenant_id, &command.room_id)
            .ok_or_else(|| GameRoomError::invalid("room is full"))?;

        if let Some(seat_index) = reusable_seat_index(&store, tenant_id, &command.room_id, seat_no)
        {
            let seat = &mut store.seats[seat_index].item;
            seat.user_id = Some(command.user_id.clone());
            seat.display_name_snapshot = command.display_name_snapshot.clone();
            seat.status = "joined".into();
            seat.version += 1;
        } else {
            store.seats.push(StoredSeat {
                tenant_id: tenant_id.into(),
                item: GameRoomSeatItem {
                    id: uuid(),
                    room_id: command.room_id.clone(),
                    seat_no,
                    team_no: None,
                    user_id: Some(command.user_id.clone()),
                    display_name_snapshot: command.display_name_snapshot.clone(),
                    status: "joined".into(),
                    version: 0,
                },
            });
        }

        refresh_room_player_count(&mut store, room_index);
        let room = &mut store.rooms[room_index].item;
        room.version += 1;
        Ok(room.clone())
    }

    async fn leave_room(
        &self,
        tenant_id: &str,
        command: &LeaveGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let room_index = find_room_index(&store, tenant_id, &command.room_id)?;
        ensure_expected_version(&store.rooms[room_index].item, command.expected_version)?;
        let seat_index =
            active_seat_index_for_user(&store, tenant_id, &command.room_id, &command.user_id)
                .ok_or_else(|| GameRoomError::not_found("active room seat not found"))?;
        let seat = &mut store.seats[seat_index].item;
        seat.status = "left".into();
        seat.version += 1;

        refresh_room_player_count(&mut store, room_index);
        let room = &mut store.rooms[room_index].item;
        room.version += 1;
        Ok(room.clone())
    }

    async fn set_ready(
        &self,
        tenant_id: &str,
        command: &ReadyGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let room_index = find_room_index(&store, tenant_id, &command.room_id)?;
        ensure_expected_version(&store.rooms[room_index].item, command.expected_version)?;
        let seat_index =
            active_seat_index_for_user(&store, tenant_id, &command.room_id, &command.user_id)
                .ok_or_else(|| GameRoomError::not_found("active room seat not found"))?;
        let seat = &mut store.seats[seat_index].item;
        seat.status = if command.ready { "ready" } else { "joined" }.into();
        seat.version += 1;

        let room = &mut store.rooms[room_index].item;
        room.version += 1;
        Ok(room.clone())
    }

    async fn start_room(
        &self,
        tenant_id: &str,
        command: &StartGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let room_index = find_room_index(&store, tenant_id, &command.room_id)?;
        ensure_expected_version(&store.rooms[room_index].item, command.expected_version)?;
        for seat in store
            .seats
            .iter_mut()
            .filter(|stored| {
                stored.tenant_id == tenant_id && stored.item.room_id == command.room_id
            })
            .filter(|stored| is_active_seat_status(&stored.item.status))
        {
            seat.item.status = "playing".into();
            seat.item.version += 1;
        }
        let room = &mut store.rooms[room_index].item;
        room.status = "in_progress".into();
        room.version += 1;
        Ok(room.clone())
    }

    async fn close_room(
        &self,
        tenant_id: &str,
        command: &CloseGameRoomCommand,
    ) -> GameRoomResult<GameRoomItem> {
        let mut store = self.store.write().map_err(lock_error)?;
        let room_index = find_room_index(&store, tenant_id, &command.room_id)?;
        ensure_expected_version(&store.rooms[room_index].item, command.expected_version)?;
        for seat in store
            .seats
            .iter_mut()
            .filter(|stored| {
                stored.tenant_id == tenant_id && stored.item.room_id == command.room_id
            })
            .filter(|stored| is_active_seat_status(&stored.item.status))
        {
            seat.item.status = "left".into();
            seat.item.version += 1;
        }
        let room = &mut store.rooms[room_index].item;
        room.status = "closed".into();
        room.current_players = 0;
        room.version += 1;
        Ok(room.clone())
    }
}

fn find_room_index(store: &RoomStore, tenant_id: &str, room_id: &str) -> GameRoomResult<usize> {
    store
        .rooms
        .iter()
        .position(|stored| {
            stored.tenant_id == tenant_id
                && (stored.item.id == room_id || stored.item.room_code == room_id)
        })
        .ok_or_else(|| GameRoomError::not_found("room not found"))
}

fn ensure_expected_version(room: &GameRoomItem, expected: Option<i64>) -> GameRoomResult<()> {
    if let Some(expected) = expected {
        if room.version != expected {
            return Err(GameRoomError::conflict("room version has changed"));
        }
    }
    Ok(())
}

fn next_available_seat_no(store: &RoomStore, tenant_id: &str, room_id: &str) -> Option<i32> {
    let room = store
        .rooms
        .iter()
        .find(|stored| stored.tenant_id == tenant_id && stored.item.id == room_id)?;
    (1..=room.item.max_players).find(|seat_no| {
        store.seats.iter().all(|stored| {
            stored.tenant_id != tenant_id
                || stored.item.room_id != room_id
                || stored.item.seat_no != *seat_no
                || !is_active_seat_status(&stored.item.status)
        })
    })
}

fn reusable_seat_index(
    store: &RoomStore,
    tenant_id: &str,
    room_id: &str,
    seat_no: i32,
) -> Option<usize> {
    store.seats.iter().position(|stored| {
        stored.tenant_id == tenant_id
            && stored.item.room_id == room_id
            && stored.item.seat_no == seat_no
            && !is_active_seat_status(&stored.item.status)
    })
}

fn active_seat_index_for_user(
    store: &RoomStore,
    tenant_id: &str,
    room_id: &str,
    user_id: &str,
) -> Option<usize> {
    store.seats.iter().position(|stored| {
        stored.tenant_id == tenant_id
            && stored.item.room_id == room_id
            && stored
                .item
                .user_id
                .as_deref()
                .is_some_and(|value| value == user_id)
            && is_active_seat_status(&stored.item.status)
    })
}

fn refresh_room_player_count(store: &mut RoomStore, room_index: usize) {
    let tenant_id = store.rooms[room_index].tenant_id.clone();
    let room_id = store.rooms[room_index].item.id.clone();
    let count = store
        .seats
        .iter()
        .filter(|stored| {
            stored.tenant_id == tenant_id
                && stored.item.room_id == room_id
                && is_active_seat_status(&stored.item.status)
        })
        .count() as i32;
    store.rooms[room_index].item.current_players = count;
}

fn is_active_seat_status(status: &str) -> bool {
    matches!(status, "reserved" | "joined" | "ready" | "playing")
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameRoomError {
    GameRoomError::invalid("room repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<GameRoomItem> {
        vec![
            GameRoomItem {
                id: "room-1".into(),
                game_id: "game-chinese-chess".into(),
                mode_id: Some("mode-ranked".into()),
                ruleset_id: Some("ruleset-v1".into()),
                room_code: "XQ-001".into(),
                host_user_id: "user-host-1".into(),
                visibility: "public".into(),
                join_policy: "open".into(),
                max_players: 2,
                current_players: 2,
                status: "in_progress".into(),
                version: 1,
            },
            GameRoomItem {
                id: "room-2".into(),
                game_id: "game-doudizhu".into(),
                mode_id: None,
                ruleset_id: None,
                room_code: "DDZ-OPEN".into(),
                host_user_id: "user-host-2".into(),
                visibility: "public".into(),
                join_policy: "open".into(),
                max_players: 4,
                current_players: 1,
                status: "open".into(),
                version: 0,
            },
        ]
    }

    #[tokio::test]
    async fn list_rooms_filters_by_status() {
        let repo = InMemoryGameRoomRepository::with_seed(sample_items());
        let page = repo
            .list_rooms(
                "100001",
                &GameRoomQuery {
                    status: Some("open".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(1, page.items.len());
        assert_eq!("open", page.items[0].status);
    }

    #[tokio::test]
    async fn list_rooms_paginates_items() {
        let repo = InMemoryGameRoomRepository::with_seed(sample_items());
        let page = repo
            .list_rooms(
                "100001",
                &GameRoomQuery {
                    page: Some(1),
                    page_size: Some(1),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(1, page.items.len());
        assert_eq!(2, page.total);
    }

    #[tokio::test]
    async fn room_lifecycle_requires_ready_players_before_start() {
        use sdkwork_game_room_service::{
            CreateGameRoomCommand, GameRoomService, JoinGameRoomCommand, ReadyGameRoomCommand,
            StartGameRoomCommand,
        };

        let service = GameRoomService::new(InMemoryGameRoomRepository::default());
        let room = service
            .create_room(
                "100001",
                CreateGameRoomCommand {
                    game_id: "game-xiangqi".into(),
                    mode_id: Some("mode-ranked".into()),
                    ruleset_id: Some("ruleset-v1".into()),
                    room_code: "ROOM-001".into(),
                    host_user_id: "user-host".into(),
                    visibility: "public".into(),
                    join_policy: "open".into(),
                    max_players: 2,
                },
            )
            .await
            .unwrap();

        assert_eq!(1, room.current_players);
        assert_eq!("open", room.status);

        let joined = service
            .join_room(
                "100001",
                JoinGameRoomCommand {
                    room_id: room.id.clone(),
                    user_id: "user-guest".into(),
                    display_name_snapshot: Some("Guest".into()),
                    expected_version: Some(room.version),
                },
            )
            .await
            .unwrap();

        assert_eq!(2, joined.current_players);

        let not_ready = service
            .start_room(
                "100001",
                StartGameRoomCommand {
                    room_id: room.id.clone(),
                    host_user_id: "user-host".into(),
                    expected_version: Some(joined.version),
                },
            )
            .await
            .unwrap_err();
        assert_eq!("invalid", not_ready.code());

        let host_ready = service
            .set_ready(
                "100001",
                ReadyGameRoomCommand {
                    room_id: room.id.clone(),
                    user_id: "user-host".into(),
                    ready: true,
                    expected_version: None,
                },
            )
            .await
            .unwrap();
        let guest_ready = service
            .set_ready(
                "100001",
                ReadyGameRoomCommand {
                    room_id: room.id.clone(),
                    user_id: "user-guest".into(),
                    ready: true,
                    expected_version: Some(host_ready.version),
                },
            )
            .await
            .unwrap();

        let started = service
            .start_room(
                "100001",
                StartGameRoomCommand {
                    room_id: room.id.clone(),
                    host_user_id: "user-host".into(),
                    expected_version: Some(guest_ready.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("in_progress", started.status);
        let seats = service.list_room_seats("100001", &room.id).await.unwrap();
        assert_eq!(2, seats.len());
        assert!(seats.iter().all(|seat| seat.status == "playing"));
    }

    #[tokio::test]
    async fn join_room_rejects_capacity_and_stale_versions() {
        use sdkwork_game_room_service::{
            CreateGameRoomCommand, GameRoomService, JoinGameRoomCommand,
        };

        let service = GameRoomService::new(InMemoryGameRoomRepository::default());
        let room = service
            .create_room(
                "100001",
                CreateGameRoomCommand {
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    ruleset_id: None,
                    room_code: "ROOM-002".into(),
                    host_user_id: "user-host".into(),
                    visibility: "public".into(),
                    join_policy: "open".into(),
                    max_players: 1,
                },
            )
            .await
            .unwrap();

        let full = service
            .join_room(
                "100001",
                JoinGameRoomCommand {
                    room_id: room.id.clone(),
                    user_id: "user-guest".into(),
                    display_name_snapshot: None,
                    expected_version: Some(room.version),
                },
            )
            .await
            .unwrap_err();
        assert_eq!("invalid", full.code());

        let stale = service
            .join_room(
                "100001",
                JoinGameRoomCommand {
                    room_id: room.id.clone(),
                    user_id: "user-host".into(),
                    display_name_snapshot: None,
                    expected_version: Some(room.version + 10),
                },
            )
            .await
            .unwrap_err();
        assert_eq!("conflict", stale.code());
    }

    #[tokio::test]
    async fn close_room_requires_host_permission() {
        use sdkwork_game_room_service::{
            CloseGameRoomCommand, CreateGameRoomCommand, GameRoomService,
        };

        let service = GameRoomService::new(InMemoryGameRoomRepository::default());
        let room = service
            .create_room(
                "100001",
                CreateGameRoomCommand {
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    ruleset_id: None,
                    room_code: "ROOM-003".into(),
                    host_user_id: "user-host".into(),
                    visibility: "private".into(),
                    join_policy: "invite".into(),
                    max_players: 2,
                },
            )
            .await
            .unwrap();

        let forbidden = service
            .close_room(
                "100001",
                CloseGameRoomCommand {
                    room_id: room.id.clone(),
                    operator_user_id: "user-guest".into(),
                    reason: Some("not mine".into()),
                    expected_version: Some(room.version),
                },
            )
            .await
            .unwrap_err();
        assert_eq!("forbidden", forbidden.code());

        let closed = service
            .close_room(
                "100001",
                CloseGameRoomCommand {
                    room_id: room.id.clone(),
                    operator_user_id: "user-host".into(),
                    reason: Some("done".into()),
                    expected_version: Some(room.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("closed", closed.status);
        assert_eq!(0, closed.current_players);
    }

    #[tokio::test]
    async fn force_close_room_allows_backend_operator() {
        use sdkwork_game_room_service::{
            CloseGameRoomCommand, CreateGameRoomCommand, GameRoomService,
        };

        let service = GameRoomService::new(InMemoryGameRoomRepository::default());
        let room = service
            .create_room(
                "100001",
                CreateGameRoomCommand {
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    ruleset_id: None,
                    room_code: "ROOM-004".into(),
                    host_user_id: "user-host".into(),
                    visibility: "private".into(),
                    join_policy: "invite".into(),
                    max_players: 2,
                },
            )
            .await
            .unwrap();

        let closed = service
            .force_close_room(
                "100001",
                CloseGameRoomCommand {
                    room_id: room.id.clone(),
                    operator_user_id: "ops-admin".into(),
                    reason: Some("abandoned room".into()),
                    expected_version: Some(room.version),
                },
            )
            .await
            .unwrap();

        assert_eq!("closed", closed.status);
        assert_eq!(0, closed.current_players);
    }
}
