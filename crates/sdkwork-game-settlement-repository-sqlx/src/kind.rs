use async_trait::async_trait;
use sdkwork_game_settlement_service::{
    CompleteSettlementJobCommand, CreateRewardIntentCommand, CreateSettlementJobCommand,
    GameRewardIntentItem, GameSettlementJobItem, GameSettlementJobPage, GameSettlementRepository,
    GameSettlementResult, RecordSettlementFailureCommand, SettlementDueJobQuery,
    StartSettlementJobCommand,
};

#[cfg(any(test, feature = "test-support"))]
use crate::memory::InMemoryGameSettlementRepository;
use crate::sqlx::SqlxGameSettlementRepository;

pub enum GameSettlementRepositoryKind {
    #[cfg(any(test, feature = "test-support"))]
    Memory(InMemoryGameSettlementRepository),
    Sqlx(Box<SqlxGameSettlementRepository>),
}

#[async_trait]
impl GameSettlementRepository for GameSettlementRepositoryKind {
    async fn create_job(
        &self,
        tenant_id: &str,
        command: &CreateSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_job(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_job(tenant_id, command).await,
        }
    }

    async fn get_job(
        &self,
        tenant_id: &str,
        job_id: &str,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.get_job(tenant_id, job_id).await,
            Self::Sqlx(repo) => repo.get_job(tenant_id, job_id).await,
        }
    }

    async fn list_due_jobs(
        &self,
        tenant_id: &str,
        query: &SettlementDueJobQuery,
    ) -> GameSettlementResult<GameSettlementJobPage> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.list_due_jobs(tenant_id, query).await,
            Self::Sqlx(repo) => repo.list_due_jobs(tenant_id, query).await,
        }
    }

    async fn start_job(
        &self,
        tenant_id: &str,
        command: &StartSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.start_job(tenant_id, command).await,
            Self::Sqlx(repo) => repo.start_job(tenant_id, command).await,
        }
    }

    async fn record_failure(
        &self,
        tenant_id: &str,
        command: &RecordSettlementFailureCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.record_failure(tenant_id, command).await,
            Self::Sqlx(repo) => repo.record_failure(tenant_id, command).await,
        }
    }

    async fn complete_job(
        &self,
        tenant_id: &str,
        command: &CompleteSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.complete_job(tenant_id, command).await,
            Self::Sqlx(repo) => repo.complete_job(tenant_id, command).await,
        }
    }

    async fn create_reward_intent(
        &self,
        tenant_id: &str,
        command: &CreateRewardIntentCommand,
    ) -> GameSettlementResult<GameRewardIntentItem> {
        match self {
            #[cfg(any(test, feature = "test-support"))]
            Self::Memory(repo) => repo.create_reward_intent(tenant_id, command).await,
            Self::Sqlx(repo) => repo.create_reward_intent(tenant_id, command).await,
        }
    }
}
