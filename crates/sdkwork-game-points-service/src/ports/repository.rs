use async_trait::async_trait;

use crate::domain::models::{
    AppendPointLedgerCommand, GamePointBalance, GamePointLedgerEntry, GamePointResult,
};

#[async_trait]
pub trait GamePointRepository: Send + Sync {
    async fn append_ledger(
        &self,
        tenant_id: &str,
        command: &AppendPointLedgerCommand,
    ) -> GamePointResult<GamePointLedgerEntry>;

    async fn get_balance(
        &self,
        tenant_id: &str,
        ledger_account_id: &str,
    ) -> GamePointResult<GamePointBalance>;
}
