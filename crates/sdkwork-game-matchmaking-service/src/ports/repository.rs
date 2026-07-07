use async_trait::async_trait;

use crate::domain::models::{
    CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingResult, MatchTicketItem,
    MatchTicketPage, MatchTicketQuery, MatchmakingQueueQuery,
};

#[async_trait]
pub trait GameMatchmakingRepository: Send + Sync {
    async fn create_ticket(
        &self,
        tenant_id: &str,
        command: &CreateMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem>;

    async fn get_ticket(
        &self,
        tenant_id: &str,
        ticket_id: &str,
    ) -> GameMatchmakingResult<MatchTicketItem>;

    async fn cancel_ticket(
        &self,
        tenant_id: &str,
        command: &CancelMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem>;

    async fn list_tickets(
        &self,
        tenant_id: &str,
        query: &MatchTicketQuery,
    ) -> GameMatchmakingResult<MatchTicketPage>;

    async fn list_queue(
        &self,
        tenant_id: &str,
        query: &MatchmakingQueueQuery,
    ) -> GameMatchmakingResult<MatchTicketPage>;
}
