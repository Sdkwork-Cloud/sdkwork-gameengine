use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    LeaderboardConfigItem, LeaderboardConfigPage, LeaderboardConfigQuery,
    LeaderboardEntriesRebuildCommand, LeaderboardEntry, LeaderboardEntryUpdateCommand,
    LeaderboardPage, LeaderboardQuery, LeaderboardResult,
};
use crate::ports::repository::LeaderboardRepository;

pub struct LeaderboardService<R> {
    repository: R,
}

impl<R> LeaderboardService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> LeaderboardService<R>
where
    R: LeaderboardRepository,
{
    pub async fn list_configs(
        &self,
        tenant_id: &str,
        query: LeaderboardConfigQuery,
    ) -> LeaderboardResult<LeaderboardConfigPage> {
        validate_required("tenant_id", tenant_id)?;
        self.repository.list_configs(tenant_id, &query).await
    }

    pub async fn get_config(
        &self,
        tenant_id: &str,
        leaderboard_id: &str,
    ) -> LeaderboardResult<LeaderboardConfigItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("leaderboard_id", leaderboard_id)?;
        self.repository.get_config(tenant_id, leaderboard_id).await
    }

    pub async fn list_rankings(
        &self,
        tenant_id: &str,
        query: LeaderboardQuery,
    ) -> LeaderboardResult<LeaderboardPage> {
        if is_blank(Some(tenant_id)) {
            return Err(crate::domain::models::LeaderboardError::invalid(
                "tenant_id is required",
            ));
        }
        self.repository.list_rankings(tenant_id, &query).await
    }

    pub async fn get_user_ranking(
        &self,
        tenant_id: &str,
        user_id: &str,
        game_id: Option<String>,
    ) -> LeaderboardResult<LeaderboardEntry> {
        if is_blank(Some(tenant_id)) {
            return Err(crate::domain::models::LeaderboardError::invalid(
                "tenant_id is required",
            ));
        }
        if is_blank(Some(user_id)) {
            return Err(crate::domain::models::LeaderboardError::invalid(
                "user_id is required",
            ));
        }
        self.repository
            .get_user_ranking(tenant_id, user_id, game_id.as_deref())
            .await
    }

    pub async fn upsert_entry(
        &self,
        tenant_id: &str,
        command: LeaderboardEntryUpdateCommand,
    ) -> LeaderboardResult<LeaderboardEntry> {
        validate_required("tenant_id", tenant_id)?;
        validate_entry_command(&command)?;
        self.repository.upsert_entry(tenant_id, &command).await
    }

    pub async fn rebuild_entries(
        &self,
        tenant_id: &str,
        command: LeaderboardEntriesRebuildCommand,
    ) -> LeaderboardResult<LeaderboardPage> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("leaderboard_id", &command.leaderboard_id)?;
        for entry in &command.entries {
            validate_entry_command(entry)?;
            if entry.leaderboard_id != command.leaderboard_id {
                return Err(crate::domain::models::LeaderboardError::conflict(
                    "rebuild entry leaderboard_id must match rebuild scope",
                ));
            }
        }
        self.repository.rebuild_entries(tenant_id, &command).await
    }
}

fn validate_required(field: &str, value: &str) -> LeaderboardResult<()> {
    if is_blank(Some(value)) {
        return Err(crate::domain::models::LeaderboardError::invalid(format!(
            "{field} is required"
        )));
    }
    Ok(())
}

fn validate_entry_command(command: &LeaderboardEntryUpdateCommand) -> LeaderboardResult<()> {
    validate_required("leaderboard_id", &command.leaderboard_id)?;
    validate_required("game_id", &command.game_id)?;
    validate_required("user_id", &command.user_id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::domain::models::{
        LeaderboardConfigItem, LeaderboardConfigPage, LeaderboardEntry, LeaderboardPage,
    };

    struct EmptyRepo;

    #[async_trait]
    impl LeaderboardRepository for EmptyRepo {
        async fn list_configs(
            &self,
            _tenant_id: &str,
            _query: &LeaderboardConfigQuery,
        ) -> LeaderboardResult<LeaderboardConfigPage> {
            Ok(LeaderboardConfigPage {
                items: vec![],
                total: 0,
                page: 1,
                page_size: 20,
            })
        }

        async fn get_config(
            &self,
            _tenant_id: &str,
            _leaderboard_id: &str,
        ) -> LeaderboardResult<LeaderboardConfigItem> {
            Err(crate::domain::models::LeaderboardError::not_found(
                "leaderboard config not found",
            ))
        }

        async fn list_rankings(
            &self,
            _tenant_id: &str,
            _query: &LeaderboardQuery,
        ) -> LeaderboardResult<LeaderboardPage> {
            Ok(LeaderboardPage {
                items: vec![],
                total: 0,
                page: 1,
                page_size: 20,
            })
        }

        async fn get_user_ranking(
            &self,
            _tenant_id: &str,
            _user_id: &str,
            _game_id: Option<&str>,
        ) -> LeaderboardResult<LeaderboardEntry> {
            Err(crate::domain::models::LeaderboardError::not_found(
                "leaderboard entry not found",
            ))
        }

        async fn upsert_entry(
            &self,
            _tenant_id: &str,
            _command: &LeaderboardEntryUpdateCommand,
        ) -> LeaderboardResult<LeaderboardEntry> {
            unreachable!("validation must reject before repository access")
        }

        async fn rebuild_entries(
            &self,
            _tenant_id: &str,
            _command: &LeaderboardEntriesRebuildCommand,
        ) -> LeaderboardResult<LeaderboardPage> {
            unreachable!("validation must reject before repository access")
        }
    }

    #[tokio::test]
    async fn list_rankings_rejects_empty_tenant() {
        let service = LeaderboardService::new(EmptyRepo);
        let result = service.list_rankings("", LeaderboardQuery::default()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "invalid");
    }

    #[tokio::test]
    async fn upsert_entry_rejects_empty_leaderboard_id() {
        let service = LeaderboardService::new(EmptyRepo);
        let result = service
            .upsert_entry(
                "100001",
                LeaderboardEntryUpdateCommand {
                    leaderboard_id: String::new(),
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    season_id: None,
                    user_id: "user-1".into(),
                    display_name_snapshot: None,
                    score_value: 10,
                    tie_breaker_value: None,
                    last_ledger_id: None,
                    recorded_at: None,
                },
            )
            .await;

        assert_eq!("invalid", result.unwrap_err().code());
    }
}
