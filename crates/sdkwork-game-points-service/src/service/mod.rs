use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    AppendPointLedgerCommand, GamePointBalance, GamePointError, GamePointLedgerEntry,
    GamePointResult,
};
use crate::ports::repository::GamePointRepository;

pub struct GamePointService<R> {
    repository: R,
}

impl<R> GamePointService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GamePointService<R>
where
    R: GamePointRepository,
{
    pub async fn append_ledger(
        &self,
        tenant_id: &str,
        command: AppendPointLedgerCommand,
    ) -> GamePointResult<GamePointLedgerEntry> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("ledger_account_id", &command.ledger_account_id)?;
        validate_required("game_id", &command.game_id)?;
        validate_required("user_id", &command.user_id)?;
        validate_required("direction", &command.direction)?;
        validate_direction(&command.direction)?;
        validate_points_delta(command.points_delta)?;
        validate_required("source_event_id", &command.source_event_id)?;
        validate_required("reason_code", &command.reason_code)?;
        validate_required("idempotency_key", &command.idempotency_key)?;

        self.repository.append_ledger(tenant_id, &command).await
    }

    pub async fn get_balance(
        &self,
        tenant_id: &str,
        ledger_account_id: &str,
    ) -> GamePointResult<GamePointBalance> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("ledger_account_id", ledger_account_id)?;

        self.repository
            .get_balance(tenant_id, ledger_account_id)
            .await
    }
}

fn validate_required(field: &str, value: &str) -> GamePointResult<()> {
    if is_blank(Some(value)) {
        return Err(GamePointError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_direction(direction: &str) -> GamePointResult<()> {
    if matches!(direction, "credit" | "debit") {
        return Ok(());
    }
    Err(GamePointError::invalid(
        "point direction must be credit or debit",
    ))
}

fn validate_points_delta(points_delta: i64) -> GamePointResult<()> {
    if points_delta <= 0 {
        return Err(GamePointError::invalid(
            "points_delta must be greater than zero",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{
        AppendPointLedgerCommand, GamePointBalance, GamePointLedgerEntry, GamePointRepository,
        GamePointResult, GamePointService,
    };

    struct EmptyRepo;

    #[async_trait]
    impl GamePointRepository for EmptyRepo {
        async fn append_ledger(
            &self,
            _tenant_id: &str,
            _command: &AppendPointLedgerCommand,
        ) -> GamePointResult<GamePointLedgerEntry> {
            unreachable!("validation must reject before repository access")
        }

        async fn get_balance(
            &self,
            _tenant_id: &str,
            _ledger_account_id: &str,
        ) -> GamePointResult<GamePointBalance> {
            unreachable!("validation must reject before repository access")
        }
    }

    #[tokio::test]
    async fn append_ledger_rejects_cash_or_wallet_direction_semantics() {
        let service = GamePointService::new(EmptyRepo);

        let result = service
            .append_ledger(
                "100001",
                AppendPointLedgerCommand {
                    ledger_account_id: "acct-game-1-user-1".into(),
                    game_id: "game-xiangqi".into(),
                    mode_id: None,
                    season_id: None,
                    user_id: "user-1".into(),
                    direction: "cash_credit".into(),
                    points_delta: 10,
                    source_event_id: "score-event-1".into(),
                    reason_code: "match_win".into(),
                    idempotency_key: "idem-1".into(),
                },
            )
            .await;

        let error = result.unwrap_err();
        assert_eq!("invalid", error.code());
    }
}
