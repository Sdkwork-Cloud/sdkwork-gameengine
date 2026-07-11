use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use sdkwork_game_room_service::{CloseGameRoomCommand, GameRoomQuery, GameRoomService};
use sdkwork_routes_games_support::{
    finish_page_response, finish_resource_response, room_page_to_list_data, StrictListQuery,
};
use sdkwork_web_axum::RequirePrincipal;
use std::sync::Arc;

pub type GamesRoomStore<R> = Arc<GameRoomService<R>>;

#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BackendRoomsListQuery {
    game_id: Option<String>,
    status: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForceCloseRoomRequest {
    reason: Option<String>,
    expected_version: Option<i64>,
}

pub fn build_room_backend_router<R>(store: GamesRoomStore<R>) -> Router
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync + 'static,
{
    Router::new()
        .route(
            crate::paths::BACKEND_GAMES_ROOMS_LIST_PATH,
            get(list_rooms::<R>),
        )
        .route(
            crate::paths::BACKEND_GAMES_ROOM_DETAIL_PATH,
            get(get_room::<R>),
        )
        .route(
            crate::paths::BACKEND_GAMES_ROOM_SEATS_LIST_PATH,
            get(list_room_seats::<R>),
        )
        .route(
            crate::paths::BACKEND_GAMES_ROOM_FORCE_CLOSE_PATH,
            post(force_close_room::<R>),
        )
        .with_state(store)
}

async fn list_rooms<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    StrictListQuery(query): StrictListQuery<BackendRoomsListQuery>,
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

async fn force_close_room<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesRoomStore<R>>,
    Path(room_id): Path<String>,
    Json(body): Json<ForceCloseRoomRequest>,
) -> Response
where
    R: sdkwork_game_room_service::GameRoomRepository + Send + Sync,
{
    finish_resource_response(
        store
            .force_close_room(
                principal.tenant_id(),
                CloseGameRoomCommand {
                    room_id,
                    operator_user_id: principal.user_id().to_owned(),
                    reason: body.reason,
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
    fn backend_room_list_query_rejects_forbidden_pagination_aliases() {
        for alias in [
            "pageSize=20",
            "limit=20",
            "page_no=1",
            "pageNo=1",
            "per_page=20",
            "size=20",
        ] {
            let uri = format!("/backend/v3/api/games/rooms?{alias}")
                .parse()
                .expect("uri");
            assert!(
                axum::extract::Query::<BackendRoomsListQuery>::try_from_uri(&uri).is_err(),
                "forbidden pagination alias must be rejected: {alias}"
            );
        }
    }
}
