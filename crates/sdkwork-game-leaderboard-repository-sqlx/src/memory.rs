use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_leaderboard_service::{
    LeaderboardConfigItem, LeaderboardConfigPage, LeaderboardConfigQuery,
    LeaderboardEntriesRebuildCommand, LeaderboardEntry, LeaderboardEntryUpdateCommand,
    LeaderboardError, LeaderboardPage, LeaderboardQuery, LeaderboardRepository, LeaderboardResult,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone, Default)]
pub struct InMemoryLeaderboardRepository {
    store: Arc<RwLock<LeaderboardStore>>,
}

impl InMemoryLeaderboardRepository {
    pub fn with_seed(items: Vec<LeaderboardEntry>) -> Self {
        Self {
            store: Arc::new(RwLock::new(LeaderboardStore {
                entries: items
                    .into_iter()
                    .map(|item| StoredEntry {
                        tenant_id: "100001".into(),
                        leaderboard_id: "default".into(),
                        item,
                    })
                    .collect(),
                configs: vec![],
            })),
        }
    }

    pub fn with_configs(configs: Vec<LeaderboardConfigItem>, items: Vec<LeaderboardEntry>) -> Self {
        Self {
            store: Arc::new(RwLock::new(LeaderboardStore {
                configs: configs
                    .into_iter()
                    .map(|item| StoredConfig {
                        tenant_id: "100001".into(),
                        item,
                    })
                    .collect(),
                entries: items
                    .into_iter()
                    .map(|item| StoredEntry {
                        tenant_id: "100001".into(),
                        leaderboard_id: "default".into(),
                        item,
                    })
                    .collect(),
            })),
        }
    }
}

#[derive(Default)]
struct LeaderboardStore {
    configs: Vec<StoredConfig>,
    entries: Vec<StoredEntry>,
}

#[derive(Clone)]
struct StoredConfig {
    tenant_id: String,
    item: LeaderboardConfigItem,
}

#[derive(Clone)]
struct StoredEntry {
    tenant_id: String,
    leaderboard_id: String,
    item: LeaderboardEntry,
}

#[async_trait]
impl LeaderboardRepository for InMemoryLeaderboardRepository {
    async fn list_configs(
        &self,
        tenant_id: &str,
        query: &LeaderboardConfigQuery,
    ) -> LeaderboardResult<LeaderboardConfigPage> {
        let store = self.store.read().map_err(lock_error)?;
        let mut filtered: Vec<LeaderboardConfigItem> = store
            .configs
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id)
            .filter(|stored| matches_config_query(&stored.item, query))
            .map(|stored| stored.item.clone())
            .collect();
        filtered.sort_by(|left, right| left.leaderboard_code.cmp(&right.leaderboard_code));
        Ok(config_page(filtered, query))
    }

    async fn get_config(
        &self,
        tenant_id: &str,
        leaderboard_id: &str,
    ) -> LeaderboardResult<LeaderboardConfigItem> {
        let store = self.store.read().map_err(lock_error)?;
        store
            .configs
            .iter()
            .find(|stored| stored.tenant_id == tenant_id && stored.item.id == leaderboard_id)
            .map(|stored| stored.item.clone())
            .ok_or_else(|| LeaderboardError::not_found("leaderboard config not found"))
    }

    async fn list_rankings(
        &self,
        tenant_id: &str,
        query: &LeaderboardQuery,
    ) -> LeaderboardResult<LeaderboardPage> {
        let store = self.store.read().map_err(lock_error)?;
        let ranked = ranked_entries(&store, tenant_id, query);
        Ok(entry_page(ranked, query))
    }

    async fn get_user_ranking(
        &self,
        tenant_id: &str,
        user_id: &str,
        game_id: Option<&str>,
    ) -> LeaderboardResult<LeaderboardEntry> {
        let store = self.store.read().map_err(lock_error)?;
        let best = store
            .entries
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id && stored.item.user_id == user_id)
            .filter(|stored| game_id.is_none_or(|game_id| stored.item.game_id == game_id))
            .max_by(|left, right| compare_entries(&left.item, &right.item))
            .map(|stored| stored.item.clone())
            .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))?;

        let query = LeaderboardQuery {
            game_id: Some(best.game_id.clone()),
            page_size: Some(200),
            ..Default::default()
        };
        let ranked = ranked_entries(&store, tenant_id, &query);
        ranked
            .into_iter()
            .find(|item| item.user_id == user_id && item.game_id == best.game_id)
            .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))
    }

    async fn upsert_entry(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntryUpdateCommand,
    ) -> LeaderboardResult<LeaderboardEntry> {
        let mut store = self.store.write().map_err(lock_error)?;
        let recorded_at = command
            .recorded_at
            .clone()
            .unwrap_or_else(|| now().to_rfc3339());
        let existing_index = store.entries.iter().position(|stored| {
            stored.tenant_id == tenant_id
                && stored.leaderboard_id == command.leaderboard_id
                && stored.item.user_id == command.user_id
        });

        let item = LeaderboardEntry {
            id: existing_index
                .map(|index| store.entries[index].item.id.clone())
                .unwrap_or_else(uuid),
            game_id: command.game_id.clone(),
            user_id: command.user_id.clone(),
            display_name: command.display_name_snapshot.clone(),
            score: command.score_value,
            rank_no: None,
            recorded_at,
        };

        if let Some(index) = existing_index {
            store.entries[index].item = item;
        } else {
            store.entries.push(StoredEntry {
                tenant_id: tenant_id.into(),
                leaderboard_id: command.leaderboard_id.clone(),
                item,
            });
        }

        recompute_ranks(&mut store, tenant_id, &command.leaderboard_id);
        store
            .entries
            .iter()
            .find(|stored| {
                stored.tenant_id == tenant_id
                    && stored.leaderboard_id == command.leaderboard_id
                    && stored.item.user_id == command.user_id
            })
            .map(|stored| stored.item.clone())
            .ok_or_else(|| LeaderboardError::not_found("leaderboard entry not found"))
    }

    async fn rebuild_entries(
        &self,
        tenant_id: &str,
        command: &LeaderboardEntriesRebuildCommand,
    ) -> LeaderboardResult<LeaderboardPage> {
        let mut store = self.store.write().map_err(lock_error)?;
        store.entries.retain(|stored| {
            !(stored.tenant_id == tenant_id && stored.leaderboard_id == command.leaderboard_id)
        });

        for entry in &command.entries {
            let recorded_at = entry
                .recorded_at
                .clone()
                .unwrap_or_else(|| now().to_rfc3339());
            store.entries.push(StoredEntry {
                tenant_id: tenant_id.into(),
                leaderboard_id: command.leaderboard_id.clone(),
                item: LeaderboardEntry {
                    id: uuid(),
                    game_id: entry.game_id.clone(),
                    user_id: entry.user_id.clone(),
                    display_name: entry.display_name_snapshot.clone(),
                    score: entry.score_value,
                    rank_no: None,
                    recorded_at,
                },
            });
        }

        recompute_ranks(&mut store, tenant_id, &command.leaderboard_id);
        let query = LeaderboardQuery {
            leaderboard_id: Some(command.leaderboard_id.clone()),
            page_size: Some(200),
            ..Default::default()
        };
        let ranked = ranked_entries(&store, tenant_id, &query);
        Ok(entry_page(ranked, &query))
    }
}

fn matches_config_query(item: &LeaderboardConfigItem, query: &LeaderboardConfigQuery) -> bool {
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

fn ranked_entries(
    store: &LeaderboardStore,
    tenant_id: &str,
    query: &LeaderboardQuery,
) -> Vec<LeaderboardEntry> {
    let mut items: Vec<LeaderboardEntry> = store
        .entries
        .iter()
        .filter(|stored| stored.tenant_id == tenant_id)
        .filter(|stored| {
            query
                .leaderboard_id
                .as_ref()
                .is_none_or(|leaderboard_id| stored.leaderboard_id == *leaderboard_id)
        })
        .filter(|stored| {
            query
                .game_id
                .as_ref()
                .is_none_or(|game_id| stored.item.game_id == *game_id)
        })
        .map(|stored| stored.item.clone())
        .collect();
    rank_items(&mut items);
    items
}

fn recompute_ranks(store: &mut LeaderboardStore, tenant_id: &str, leaderboard_id: &str) {
    let mut entries: Vec<(usize, LeaderboardEntry)> = store
        .entries
        .iter()
        .enumerate()
        .filter(|(_, stored)| {
            stored.tenant_id == tenant_id && stored.leaderboard_id == leaderboard_id
        })
        .map(|(index, stored)| (index, stored.item.clone()))
        .collect();
    entries.sort_by(|left, right| compare_entries(&left.1, &right.1));
    for (rank_index, (store_index, _)) in entries.into_iter().enumerate() {
        store.entries[store_index].item.rank_no = Some((rank_index + 1) as i32);
    }
}

fn entry_page(items: Vec<LeaderboardEntry>, query: &LeaderboardQuery) -> LeaderboardPage {
    let total = items.len() as u64;
    let offset = query.offset() as usize;
    let limit = query.limit() as usize;
    LeaderboardPage {
        items: items.into_iter().skip(offset).take(limit).collect(),
        total,
        page: query.page.unwrap_or(1),
        page_size: query.limit(),
    }
}

fn config_page(
    items: Vec<LeaderboardConfigItem>,
    query: &LeaderboardConfigQuery,
) -> LeaderboardConfigPage {
    let total = items.len() as u64;
    let offset = query.offset() as usize;
    let limit = query.limit() as usize;
    LeaderboardConfigPage {
        items: items.into_iter().skip(offset).take(limit).collect(),
        total,
        page: query.page.unwrap_or(1),
        page_size: query.limit(),
    }
}

fn rank_items(items: &mut [LeaderboardEntry]) {
    items.sort_by(compare_entries);
    for (index, item) in items.iter_mut().enumerate() {
        item.rank_no = Some((index + 1) as i32);
    }
}

fn compare_entries(left: &LeaderboardEntry, right: &LeaderboardEntry) -> std::cmp::Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.recorded_at.cmp(&right.recorded_at))
        .then_with(|| left.id.cmp(&right.id))
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> LeaderboardError {
    LeaderboardError::invalid("leaderboard repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_command(user_id: &str, score_value: i64) -> LeaderboardEntryUpdateCommand {
        LeaderboardEntryUpdateCommand {
            leaderboard_id: "leaderboard-ranked".into(),
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            season_id: Some("season-2026".into()),
            user_id: user_id.into(),
            display_name_snapshot: Some(user_id.into()),
            score_value,
            tie_breaker_value: None,
            last_ledger_id: Some(format!("ledger-{user_id}-{score_value}")),
            recorded_at: Some("2026-07-07T00:00:00Z".into()),
        }
    }

    #[tokio::test]
    async fn upsert_entry_updates_score_and_rebuilds_rank_order() {
        let repo = InMemoryLeaderboardRepository::default();

        repo.upsert_entry("100001", &entry_command("user-a", 10))
            .await
            .unwrap();
        repo.upsert_entry("100001", &entry_command("user-b", 20))
            .await
            .unwrap();

        let page = repo
            .list_rankings(
                "100001",
                &LeaderboardQuery {
                    leaderboard_id: Some("leaderboard-ranked".into()),
                    page_size: Some(10),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!("user-b", page.items[0].user_id);
        assert_eq!(Some(1), page.items[0].rank_no);
        assert_eq!("user-a", page.items[1].user_id);
        assert_eq!(Some(2), page.items[1].rank_no);

        repo.upsert_entry("100001", &entry_command("user-a", 30))
            .await
            .unwrap();
        let updated = repo
            .list_rankings(
                "100001",
                &LeaderboardQuery {
                    leaderboard_id: Some("leaderboard-ranked".into()),
                    page_size: Some(10),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!("user-a", updated.items[0].user_id);
        assert_eq!(30, updated.items[0].score);
        assert_eq!(Some(1), updated.items[0].rank_no);
    }

    #[tokio::test]
    async fn rebuild_entries_replaces_leaderboard_scope() {
        let repo = InMemoryLeaderboardRepository::default();
        repo.rebuild_entries(
            "100001",
            &LeaderboardEntriesRebuildCommand {
                leaderboard_id: "leaderboard-ranked".into(),
                entries: vec![entry_command("user-a", 10), entry_command("user-b", 20)],
            },
        )
        .await
        .unwrap();
        repo.rebuild_entries(
            "100001",
            &LeaderboardEntriesRebuildCommand {
                leaderboard_id: "leaderboard-ranked".into(),
                entries: vec![entry_command("user-c", 50)],
            },
        )
        .await
        .unwrap();

        let page = repo
            .list_rankings(
                "100001",
                &LeaderboardQuery {
                    leaderboard_id: Some("leaderboard-ranked".into()),
                    page_size: Some(10),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(1, page.total);
        assert_eq!("user-c", page.items[0].user_id);
        assert_eq!(Some(1), page.items[0].rank_no);
    }

    #[tokio::test]
    async fn list_configs_filters_by_game_status_and_tenant() {
        let repo = InMemoryLeaderboardRepository::with_configs(
            vec![
                LeaderboardConfigItem {
                    id: "leaderboard-ranked".into(),
                    game_id: "game-xiangqi".into(),
                    mode_id: Some("mode-ranked".into()),
                    season_id: Some("season-2026".into()),
                    leaderboard_code: "ranked-season".into(),
                    title: "Ranked Season".into(),
                    status: "active".into(),
                    ranking_metric: "points".into(),
                    ranking_order: "desc".into(),
                    tie_breaker: "updated_at".into(),
                    version: 0,
                },
                LeaderboardConfigItem {
                    id: "leaderboard-draft".into(),
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    season_id: None,
                    leaderboard_code: "draft".into(),
                    title: "Draft".into(),
                    status: "draft".into(),
                    ranking_metric: "points".into(),
                    ranking_order: "desc".into(),
                    tie_breaker: "updated_at".into(),
                    version: 0,
                },
            ],
            vec![],
        );

        let page = repo
            .list_configs(
                "100001",
                &LeaderboardConfigQuery {
                    game_id: Some("game-xiangqi".into()),
                    status: Some("active".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(1, page.total);
        assert_eq!("leaderboard-ranked", page.items[0].id);

        let other_tenant = repo
            .list_configs(
                "200001",
                &LeaderboardConfigQuery {
                    game_id: Some("game-xiangqi".into()),
                    status: Some("active".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(0, other_tenant.total);
    }
}
