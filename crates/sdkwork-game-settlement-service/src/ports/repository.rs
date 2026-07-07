use async_trait::async_trait;

use crate::domain::models::{
    CompleteSettlementJobCommand, CreateRewardIntentCommand, CreateSettlementJobCommand,
    GameRewardIntentItem, GameSettlementJobItem, GameSettlementJobPage, GameSettlementResult,
    RecordSettlementFailureCommand, SettlementDueJobQuery, StartSettlementJobCommand,
};

#[async_trait]
pub trait GameSettlementRepository: Send + Sync {
    async fn create_job(
        &self,
        tenant_id: &str,
        command: &CreateSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem>;

    async fn get_job(
        &self,
        tenant_id: &str,
        job_id: &str,
    ) -> GameSettlementResult<GameSettlementJobItem>;

    async fn list_due_jobs(
        &self,
        tenant_id: &str,
        query: &SettlementDueJobQuery,
    ) -> GameSettlementResult<GameSettlementJobPage>;

    async fn start_job(
        &self,
        tenant_id: &str,
        command: &StartSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem>;

    async fn record_failure(
        &self,
        tenant_id: &str,
        command: &RecordSettlementFailureCommand,
    ) -> GameSettlementResult<GameSettlementJobItem>;

    async fn complete_job(
        &self,
        tenant_id: &str,
        command: &CompleteSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem>;

    async fn create_reward_intent(
        &self,
        tenant_id: &str,
        command: &CreateRewardIntentCommand,
    ) -> GameSettlementResult<GameRewardIntentItem>;
}
