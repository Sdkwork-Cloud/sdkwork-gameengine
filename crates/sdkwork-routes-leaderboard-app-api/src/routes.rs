use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_game_leaderboard_service::{LeaderboardQuery, LeaderboardService};
use sdkwork_routes_games_support::{
    finish_page_response, finish_resource_response, leaderboard_page_to_list_data,
};
use sdkwork_web_axum::RequirePrincipal;
use std::sync::Arc;

pub type GamesLeaderboardStore<R> = Arc<LeaderboardService<R>>;

#[derive(Debug, serde::Deserialize, Default)]
pub struct GamesLeaderboardListQuery {
    game_id: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
}

pub fn build_leaderboard_app_router<R>(store: GamesLeaderboardStore<R>) -> Router
where
    R: sdkwork_game_leaderboard_service::LeaderboardRepository + Send + Sync + 'static,
{
    Router::new()
        .route(
            crate::paths::GAMES_LEADERBOARD_ME_PATH,
            get(get_my_leaderboard_ranking::<R>),
        )
        .route(
            crate::paths::GAMES_LEADERBOARD_LIST_PATH,
            get(list_leaderboard::<R>),
        )
        .with_state(store)
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct GamesLeaderboardMeQuery {
    game_id: Option<String>,
}

async fn list_leaderboard<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesLeaderboardStore<R>>,
    Query(query): Query<GamesLeaderboardListQuery>,
) -> Response
where
    R: sdkwork_game_leaderboard_service::LeaderboardRepository + Send + Sync,
{
    let leaderboard_query = LeaderboardQuery {
        game_id: query.game_id,
        page: query.page,
        page_size: query.page_size,
        ..Default::default()
    };

    finish_page_response(
        store
            .list_rankings(principal.tenant_id(), leaderboard_query)
            .await
            .map(leaderboard_page_to_list_data),
    )
}

async fn get_my_leaderboard_ranking<R>(
    RequirePrincipal(principal): RequirePrincipal,
    State(store): State<GamesLeaderboardStore<R>>,
    Query(query): Query<GamesLeaderboardMeQuery>,
) -> Response
where
    R: sdkwork_game_leaderboard_service::LeaderboardRepository + Send + Sync,
{
    finish_resource_response(
        store
            .get_user_ranking(principal.tenant_id(), principal.user_id(), query.game_id)
            .await,
    )
}
