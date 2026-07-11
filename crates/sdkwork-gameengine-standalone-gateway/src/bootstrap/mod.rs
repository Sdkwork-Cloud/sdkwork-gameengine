mod catalog;
mod leaderboard;
mod room;
mod routers;

use sdkwork_games_database_host::bootstrap_games_database_from_env;

pub use catalog::build_catalog_service;
use catalog::SharedCatalogService;
pub use leaderboard::build_leaderboard_service;
use leaderboard::SharedLeaderboardService;
pub use room::build_room_service;
use room::SharedRoomService;
pub use routers::build_router;

pub struct GatewayServices {
    pub catalog: SharedCatalogService,
    pub leaderboard: SharedLeaderboardService,
    pub room: SharedRoomService,
}

pub async fn build_gateway_services() -> Result<GatewayServices, String> {
    let host = bootstrap_games_database_from_env().await?;
    Ok(GatewayServices {
        catalog: build_catalog_service(&host),
        leaderboard: build_leaderboard_service(&host),
        room: build_room_service(&host),
    })
}
