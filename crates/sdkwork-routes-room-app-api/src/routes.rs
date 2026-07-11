use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use sdkwork_game_room_service::{
    CloseGameRoomCommand, CreateGameRoomCommand, GameRoomQuery, GameRoomService,
    JoinGameRoomCommand, LeaveGameRoomCommand, ReadyGameRoomCommand, StartGameRoomCommand,
};
use sdkwork_routes_games_support::{
    finish_created_resource_response, finish_page_response, finish_resource_response,
    room_page_to_list_data, StrictListQuery,
};
use sdkwork_web_axum::RequirePrincipal;
use std::sync::Arc;

pub type GamesRoomStore<R> = Arc<GameRoomService<R>>;

#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GamesRoomsListQuery {
    game_id: Option<String>,
    status: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomRequest {
    game_id: String,
    mode_id: Option<String>,
    ruleset_id: Option<String>,
    room_code: String,
    visibility: Option<String>,
    join_policy: Option<String>,
    max_players: i32,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomRequest {
    display_name_snapshot: Option<String>,
    expected_version: Option<i64>,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedVersionRequest {
    expected_version: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyRoomRequest {
    ready: bool,
    expected_version: Option<i64>,
}

pub fn build_room_app_router<R>(store: GamesRoomStore<R>) -> Router
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync + 'static,
{
    Router::new()
        .route(
            crate::paths::GAMES_ROOMS_LIST_PATH,
            get(list_rooms::<R>).post(create_room::<R>),
        )
        .route(crate::paths::GAMES_ROOM_DETAIL_PATH, get(get_room::<R>))
        .route(
            crate::paths::GAMES_ROOM_SEATS_LIST_PATH,
            get(list_room_seats::<R>),
        )
        .route(crate::paths::GAMES_ROOM_JOIN_PATH, post(join_room::<R>))
        .route(crate::paths::GAMES_ROOM_LEAVE_PATH, post(leave_room::<R>))
        .route(crate::paths::GAMES_ROOM_READY_PATH, post(set_ready::<R>))
        .route(crate::paths::GAMES_ROOM_START_PATH, post(start_room::<R>))
        .route(crate::paths::GAMES_ROOM_CLOSE_PATH, post(close_room::<R>))
        .with_state(store)
}

async fn list_rooms<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    StrictListQuery(query): StrictListQuery<GamesRoomsListQuery>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    let room_query = GameRoomQuery {
        game_id: query.game_id,
        status: query.status,
        page: query.page,
        page_size: query.page_size,
    };

    finish_page_response(
        store
            .list_rooms(principal.tenant_id(), room_query)
            .await
            .map(room_page_to_list_data),
    )
}

async fn create_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Json(body): Json<CreateRoomRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_created_resource_response(
        store
            .create_room(
                principal.tenant_id(),
                CreateGameRoomCommand {
                    game_id: body.game_id,
                    mode_id: body.mode_id,
                    ruleset_id: body.ruleset_id,
                    room_code: body.room_code,
                    host_user_id: principal.user_id().to_owned(),
                    visibility: body.visibility.unwrap_or_else(|| "public".into()),
                    join_policy: body.join_policy.unwrap_or_else(|| "open".into()),
                    max_players: body.max_players,
                },
            )
            .await,
    )
}

async fn get_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(store.get_room(principal.tenant_id(), &room_id).await)
}

async fn list_room_seats<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_page_response(
        store
            .list_room_seats(principal.tenant_id(), &room_id)
            .await
            .map(|items| {
                let total = items.len() as i64;
                sdkwork_utils_rust::offset_list_page_data(
                    items,
                    total,
                    sdkwork_utils_rust::OffsetListPageParams::parse(Some(1), Some(200)),
                )
            }),
    )
}

async fn join_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
    Json(body): Json<JoinRoomRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(
        store
            .join_room(
                principal.tenant_id(),
                JoinGameRoomCommand {
                    room_id,
                    user_id: principal.user_id().to_owned(),
                    display_name_snapshot: body.display_name_snapshot,
                    expected_version: body.expected_version,
                },
            )
            .await,
    )
}

async fn leave_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
    Json(body): Json<ExpectedVersionRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(
        store
            .leave_room(
                principal.tenant_id(),
                LeaveGameRoomCommand {
                    room_id,
                    user_id: principal.user_id().to_owned(),
                    expected_version: body.expected_version,
                },
            )
            .await,
    )
}

async fn set_ready<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
    Json(body): Json<ReadyRoomRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(
        store
            .set_ready(
                principal.tenant_id(),
                ReadyGameRoomCommand {
                    room_id,
                    user_id: principal.user_id().to_owned(),
                    ready: body.ready,
                    expected_version: body.expected_version,
                },
            )
            .await,
    )
}

async fn start_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
    Json(body): Json<ExpectedVersionRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(
        store
            .start_room(
                principal.tenant_id(),
                StartGameRoomCommand {
                    room_id,
                    host_user_id: principal.user_id().to_owned(),
                    expected_version: body.expected_version,
                },
            )
            .await,
    )
}

async fn close_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
    Json(body): Json<ExpectedVersionRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(
        store
            .close_room(
                principal.tenant_id(),
                CloseGameRoomCommand {
                    room_id,
                    operator_user_id: principal.user_id().to_owned(),
                    reason: None,
                    expected_version: body.expected_version,
                },
            )
            .await,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_list_query_rejects_forbidden_pagination_aliases() {
        for alias in [
            "pageSize=20",
            "limit=20",
            "page_no=1",
            "pageNo=1",
            "per_page=20",
            "size=20",
        ] {
            let uri = format!("/app/v3/api/games/rooms?{alias}")
                .parse()
                .expect("uri");
            assert!(
                axum::extract::Query::<GamesRoomsListQuery>::try_from_uri(&uri).is_err(),
                "forbidden pagination alias must be rejected: {alias}"
            );
        }
    }
}
