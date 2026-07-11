use sdkwork_utils_rust::string::is_blank;
use sdkwork_utils_rust::validated_offset_list_params;

use crate::domain::models::{
    CompleteSettlementJobCommand, CreateRewardIntentCommand, CreateSettlementJobCommand,
    GameRewardIntentItem, GameSettlementError, GameSettlementJobItem, GameSettlementJobPage,
    GameSettlementResult, RecordSettlementFailureCommand, SettlementDueJobQuery,
    StartSettlementJobCommand,
};
use crate::ports::repository::GameSettlementRepository;

pub struct GameSettlementService<R> {
    repository: R,
}

impl<R> GameSettlementService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameSettlementService<R>
where
    R: GameSettlementRepository,
{
    pub async fn create_job(
        &self,
        tenant_id: &str,
        command: CreateSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("session_id", &command.session_id)?;
        validate_required("session_result_id", &command.session_result_id)?;
        validate_required("idempotency_key", &command.idempotency_key)?;
        if command.job_payload.is_null() {
            return Err(GameSettlementError::invalid("job_payload must not be null"));
        }
        self.repository.create_job(tenant_id, &command).await
    }

    pub async fn get_job(
        &self,
        tenant_id: &str,
        job_id: &str,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("job_id", job_id)?;
        self.repository.get_job(tenant_id, job_id).await
    }

    pub async fn list_due_jobs(
        &self,
        tenant_id: &str,
        query: SettlementDueJobQuery,
    ) -> GameSettlementResult<GameSettlementJobPage> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("due_at", &query.due_at)?;
        validate_pagination(query.page, query.page_size)?;
        self.repository.list_due_jobs(tenant_id, &query).await
    }

    pub async fn start_job(
        &self,
        tenant_id: &str,
        command: StartSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("job_id", &command.job_id)?;
        self.repository.start_job(tenant_id, &command).await
    }

    pub async fn record_failure(
        &self,
        tenant_id: &str,
        command: RecordSettlementFailureCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("job_id", &command.job_id)?;
        validate_required("error_code", &command.error_code)?;
        self.repository.record_failure(tenant_id, &command).await
    }

    pub async fn complete_job(
        &self,
        tenant_id: &str,
        command: CompleteSettlementJobCommand,
    ) -> GameSettlementResult<GameSettlementJobItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("job_id", &command.job_id)?;
        self.repository.complete_job(tenant_id, &command).await
    }

    pub async fn create_reward_intent(
        &self,
        tenant_id: &str,
        command: CreateRewardIntentCommand,
    ) -> GameSettlementResult<GameRewardIntentItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("settlement_job_id", &command.settlement_job_id)?;
        validate_required("user_id", &command.user_id)?;
        validate_required("reward_type", &command.reward_type)?;
        validate_reward_type(&command.reward_type)?;
        validate_required("external_owner", &command.external_owner)?;
        validate_external_owner(&command.external_owner)?;
        validate_required("idempotency_key", &command.idempotency_key)?;
        if command.intent_payload.is_null() {
            return Err(GameSettlementError::invalid(
                "intent_payload must not be null",
            ));
        }
        self.repository
            .create_reward_intent(tenant_id, &command)
            .await
    }
}

fn validate_required(field: &str, value: &str) -> GameSettlementResult<()> {
    if is_blank(Some(value)) {
        return Err(GameSettlementError::invalid(format!("{field} is required")));
    }
    Ok(())
}

fn validate_pagination(page: Option<u32>, page_size: Option<u32>) -> GameSettlementResult<()> {
    validated_offset_list_params(page.map(i64::from), page_size.map(i64::from))
        .map(|_| ())
        .map_err(|_| {
            GameSettlementError::invalid_parameter(
                "page and page_size must follow SDKWork pagination bounds",
            )
        })
}

fn validate_reward_type(reward_type: &str) -> GameSettlementResult<()> {
    if matches!(
        reward_type,
        "points" | "wallet_credit" | "coupon" | "item" | "entitlement"
    ) {
        return Ok(());
    }
    Err(GameSettlementError::invalid(
        "reward_type is not supported by game settlement",
    ))
}

fn validate_external_owner(external_owner: &str) -> GameSettlementResult<()> {
    if matches!(
        external_owner,
        "game" | "commerce" | "wallet" | "inventory" | "entitlement"
    ) {
        return Ok(());
    }
    Err(GameSettlementError::invalid(
        "external_owner is not supported by game settlement",
    ))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;

    struct EmptyRepo;

    #[async_trait]
    impl GameSettlementRepository for EmptyRepo {
        async fn create_job(
            &self,
            _tenant_id: &str,
            command: &CreateSettlementJobCommand,
        ) -> GameSettlementResult<GameSettlementJobItem> {
            Ok(job_item(command))
        }

        async fn get_job(
            &self,
            _tenant_id: &str,
            _job_id: &str,
        ) -> GameSettlementResult<GameSettlementJobItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn list_due_jobs(
            &self,
            _tenant_id: &str,
            _query: &SettlementDueJobQuery,
        ) -> GameSettlementResult<GameSettlementJobPage> {
            unreachable!("validation must reject before repository access")
        }

        async fn start_job(
            &self,
            _tenant_id: &str,
            _command: &StartSettlementJobCommand,
        ) -> GameSettlementResult<GameSettlementJobItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn record_failure(
            &self,
            _tenant_id: &str,
            _command: &RecordSettlementFailureCommand,
        ) -> GameSettlementResult<GameSettlementJobItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn complete_job(
            &self,
            _tenant_id: &str,
            _command: &CompleteSettlementJobCommand,
        ) -> GameSettlementResult<GameSettlementJobItem> {
            unreachable!("validation must reject before repository access")
        }

        async fn create_reward_intent(
            &self,
            _tenant_id: &str,
            command: &CreateRewardIntentCommand,
        ) -> GameSettlementResult<GameRewardIntentItem> {
            Ok(reward_item(command))
        }
    }

    fn create_job_command() -> CreateSettlementJobCommand {
        CreateSettlementJobCommand {
            session_id: "session-1".into(),
            session_result_id: "result-1".into(),
            idempotency_key: "settlement-session-1-result-1".into(),
            job_payload: json!({"source": "session_result"}),
        }
    }

    fn reward_command(reward_type: &str, external_owner: &str) -> CreateRewardIntentCommand {
        CreateRewardIntentCommand {
            settlement_job_id: "job-1".into(),
            user_id: "user-1".into(),
            reward_type: reward_type.into(),
            external_owner: external_owner.into(),
            intent_payload: json!({"amount": 30}),
            idempotency_key: "reward-job-1-user-1".into(),
        }
    }

    fn job_item(command: &CreateSettlementJobCommand) -> GameSettlementJobItem {
        GameSettlementJobItem {
            id: "job-1".into(),
            session_id: command.session_id.clone(),
            session_result_id: command.session_result_id.clone(),
            status: "pending".into(),
            attempt_count: 0,
            idempotency_key: command.idempotency_key.clone(),
            error_code: None,
            error_detail: None,
            job_payload: command.job_payload.clone(),
            created_at: "2026-07-07T00:00:00Z".into(),
            started_at: None,
            completed_at: None,
            next_retry_at: None,
            version: 0,
        }
    }

    fn reward_item(command: &CreateRewardIntentCommand) -> GameRewardIntentItem {
        GameRewardIntentItem {
            id: "intent-1".into(),
            settlement_job_id: command.settlement_job_id.clone(),
            user_id: command.user_id.clone(),
            reward_type: command.reward_type.clone(),
            external_owner: command.external_owner.clone(),
            external_reference_id: None,
            intent_payload: command.intent_payload.clone(),
            status: "pending".into(),
            idempotency_key: command.idempotency_key.clone(),
            created_at: "2026-07-07T00:00:00Z".into(),
            submitted_at: None,
            completed_at: None,
            version: 0,
        }
    }

    #[tokio::test]
    async fn create_job_rejects_null_payload() {
        let service = GameSettlementService::new(EmptyRepo);
        let mut command = create_job_command();
        command.job_payload = serde_json::Value::Null;

        let error = service
            .create_job("100001", command)
            .await
            .expect_err("null payload must fail");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn reward_intent_rejects_direct_wallet_write_semantics() {
        let service = GameSettlementService::new(EmptyRepo);

        let error = service
            .create_reward_intent("100001", reward_command("wallet_balance_write", "wallet"))
            .await
            .expect_err("direct wallet writes must not be a game settlement reward type");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn reward_intent_accepts_external_wallet_credit_intent() {
        let service = GameSettlementService::new(EmptyRepo);

        let intent = service
            .create_reward_intent("100001", reward_command("wallet_credit", "wallet"))
            .await
            .expect("intent");

        assert_eq!("wallet_credit", intent.reward_type);
        assert_eq!("wallet", intent.external_owner);
        assert_eq!("pending", intent.status);
    }

    #[tokio::test]
    async fn due_job_query_rejects_invalid_pagination_before_repository_access() {
        let service = GameSettlementService::new(EmptyRepo);

        let page_size_error = service
            .list_due_jobs(
                "100001",
                SettlementDueJobQuery {
                    due_at: "2026-07-07T00:00:00Z".into(),
                    page: Some(1),
                    page_size: Some(201),
                },
            )
            .await
            .expect_err("page_size above the SDKWork maximum must fail");
        assert_eq!("invalid_parameter", page_size_error.code());
        assert_eq!(
            "page and page_size must follow SDKWork pagination bounds",
            page_size_error.message()
        );

        let page_error = service
            .list_due_jobs(
                "100001",
                SettlementDueJobQuery {
                    due_at: "2026-07-07T00:00:00Z".into(),
                    page: Some(0),
                    page_size: Some(20),
                },
            )
            .await
            .expect_err("page zero must fail");
        assert_eq!("invalid_parameter", page_error.code());
        assert_eq!(
            "page and page_size must follow SDKWork pagination bounds",
            page_error.message()
        );
    }

    #[tokio::test]
    async fn due_job_query_clamps_page_size() {
        let query = SettlementDueJobQuery {
            due_at: "2026-07-07T00:00:00Z".into(),
            page: Some(2),
            page_size: Some(500),
        };

        assert_eq!(200, query.limit());
        assert_eq!(200, query.offset());
    }
}
