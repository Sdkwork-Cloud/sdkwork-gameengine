use sdkwork_game_leaderboard_service::{LeaderboardEntry, LeaderboardPage};
use sdkwork_utils_rust::{offset_list_page_data, OffsetListPageParams, SdkWorkPageData};

pub fn leaderboard_page_to_list_data(page: LeaderboardPage) -> SdkWorkPageData<LeaderboardEntry> {
    let params =
        OffsetListPageParams::parse(Some(i64::from(page.page)), Some(i64::from(page.page_size)));
    offset_list_page_data(page.items, page.total as i64, params)
}
