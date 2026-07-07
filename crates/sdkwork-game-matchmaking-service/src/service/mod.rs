use sdkwork_utils_rust::string::is_blank;

use crate::domain::models::{
    CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingError,
    GameMatchmakingResult, MatchTicketItem, MatchTicketPage, MatchTicketQuery,
    MatchmakingQueueQuery,
};
use crate::ports::repository::GameMatchmakingRepository;

pub struct GameMatchmakingService<R> {
    repository: R,
}

impl<R> GameMatchmakingService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> GameMatchmakingService<R>
where
    R: GameMatchmakingRepository,
{
    pub async fn create_ticket(
        &self,
        tenant_id: &str,
        command: CreateMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("game_id", &command.game_id)?;
        validate_required("user_id", &command.user_id)?;
        validate_required("idempotency_key", &command.idempotency_key)?;
        if command.priority < 0 {
            return Err(GameMatchmakingError::invalid(
                "match ticket priority must be greater than or equal to zero",
            ));
        }
        self.repository.create_ticket(tenant_id, &command).await
    }

    pub async fn get_ticket(
        &self,
        tenant_id: &str,
        ticket_id: &str,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("ticket_id", ticket_id)?;
        self.repository.get_ticket(tenant_id, ticket_id).await
    }

    pub async fn cancel_ticket(
        &self,
        tenant_id: &str,
        command: CancelMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("ticket_id", &command.ticket_id)?;
        validate_required("user_id", &command.user_id)?;
        validate_required("reason", &command.reason)?;
        self.repository.cancel_ticket(tenant_id, &command).await
    }

    pub async fn list_tickets(
        &self,
        tenant_id: &str,
        query: MatchTicketQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        validate_required("tenant_id", tenant_id)?;
        self.repository.list_tickets(tenant_id, &query).await
    }

    pub async fn list_queue(
        &self,
        tenant_id: &str,
        query: MatchmakingQueueQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        validate_required("tenant_id", tenant_id)?;
        validate_required("game_id", &query.game_id)?;
        self.repository.list_queue(tenant_id, &query).await
    }
}

fn validate_required(field: &str, value: &str) -> GameMatchmakingResult<()> {
    if is_blank(Some(value)) {
        return Err(GameMatchmakingError::invalid(format!(
            "{field} is required"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;
    use crate::domain::models::GameMatchmakingError;

    struct EmptyRepo;

    #[async_trait]
    impl GameMatchmakingRepository for EmptyRepo {
        async fn create_ticket(
            &self,
            _tenant_id: &str,
            command: &CreateMatchTicketCommand,
        ) -> GameMatchmakingResult<MatchTicketItem> {
            Ok(ticket_item(command))
        }

        async fn get_ticket(
            &self,
            _tenant_id: &str,
            _ticket_id: &str,
        ) -> GameMatchmakingResult<MatchTicketItem> {
            Err(GameMatchmakingError::not_found("match ticket not found"))
        }

        async fn cancel_ticket(
            &self,
            _tenant_id: &str,
            _command: &CancelMatchTicketCommand,
        ) -> GameMatchmakingResult<MatchTicketItem> {
            Err(GameMatchmakingError::invalid("not implemented"))
        }

        async fn list_tickets(
            &self,
            _tenant_id: &str,
            query: &MatchTicketQuery,
        ) -> GameMatchmakingResult<MatchTicketPage> {
            Ok(MatchTicketPage {
                items: vec![],
                total: 0,
                page: query.page.unwrap_or(1),
                page_size: query.limit(),
            })
        }

        async fn list_queue(
            &self,
            _tenant_id: &str,
            query: &MatchmakingQueueQuery,
        ) -> GameMatchmakingResult<MatchTicketPage> {
            Ok(MatchTicketPage {
                items: vec![],
                total: 0,
                page: query.page.unwrap_or(1),
                page_size: query.limit(),
            })
        }
    }

    fn command(idempotency_key: &str) -> CreateMatchTicketCommand {
        CreateMatchTicketCommand {
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            ruleset_id: Some("ruleset-standard".into()),
            user_id: "user-1".into(),
            party_id: None,
            priority: 10,
            match_attributes: json!({"rank": "gold"}),
            idempotency_key: idempotency_key.into(),
            expires_at: None,
        }
    }

    fn ticket_item(command: &CreateMatchTicketCommand) -> MatchTicketItem {
        MatchTicketItem {
            id: "ticket-1".into(),
            ticket_code: "MT-1".into(),
            game_id: command.game_id.clone(),
            mode_id: command.mode_id.clone(),
            ruleset_id: command.ruleset_id.clone(),
            user_id: command.user_id.clone(),
            party_id: command.party_id.clone(),
            status: "queued".into(),
            priority: command.priority,
            match_attributes: command.match_attributes.clone(),
            idempotency_key: command.idempotency_key.clone(),
            queued_at: "2026-07-07T00:00:00Z".into(),
            matched_at: None,
            cancelled_at: None,
            expires_at: command.expires_at.clone(),
            version: 0,
        }
    }

    #[test]
    fn match_ticket_query_clamps_page_size() {
        let query = MatchTicketQuery {
            page: Some(2),
            page_size: Some(500),
            ..Default::default()
        };

        assert_eq!(200, query.limit());
        assert_eq!(200, query.offset());
    }

    #[tokio::test]
    async fn create_ticket_rejects_missing_idempotency_key() {
        let service = GameMatchmakingService::new(EmptyRepo);
        let mut command = command("");
        command.idempotency_key = " ".into();

        let error = service
            .create_ticket("100001", command)
            .await
            .expect_err("missing idempotency key must fail");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn cancel_ticket_rejects_empty_reason() {
        let service = GameMatchmakingService::new(EmptyRepo);

        let error = service
            .cancel_ticket(
                "100001",
                CancelMatchTicketCommand {
                    ticket_id: "ticket-1".into(),
                    user_id: "user-1".into(),
                    reason: " ".into(),
                },
            )
            .await
            .expect_err("empty cancel reason must fail");

        assert_eq!("invalid", error.code());
    }

    #[tokio::test]
    async fn create_ticket_accepts_valid_queued_ticket() {
        let service = GameMatchmakingService::new(EmptyRepo);

        let item = service
            .create_ticket("100001", command("idem-1"))
            .await
            .expect("ticket");

        assert_eq!("queued", item.status);
        assert_eq!("game-xiangqi", item.game_id);
    }
}
