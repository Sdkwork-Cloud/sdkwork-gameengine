use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_matchmaking_service::{
    CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingError,
    GameMatchmakingRepository, GameMatchmakingResult, MatchTicketItem, MatchTicketPage,
    MatchTicketQuery, MatchmakingQueueQuery,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone, Default)]
pub struct InMemoryGameMatchmakingRepository {
    tickets: Arc<RwLock<Vec<StoredTicket>>>,
}

#[derive(Clone)]
struct StoredTicket {
    tenant_id: String,
    item: MatchTicketItem,
}

#[async_trait]
impl GameMatchmakingRepository for InMemoryGameMatchmakingRepository {
    async fn create_ticket(
        &self,
        tenant_id: &str,
        command: &CreateMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        let mut tickets = self.tickets.write().map_err(lock_error)?;
        if let Some(existing) = tickets.iter().find(|stored| {
            stored.tenant_id == tenant_id && stored.item.idempotency_key == command.idempotency_key
        }) {
            ensure_idempotent_replay(&existing.item, command)?;
            return Ok(existing.item.clone());
        }

        let timestamp = now().to_rfc3339();
        let id = uuid();
        let item = MatchTicketItem {
            id: id.clone(),
            ticket_code: format!("MT-{id}"),
            game_id: command.game_id.clone(),
            mode_id: command.mode_id.clone(),
            ruleset_id: command.ruleset_id.clone(),
            user_id: command.user_id.clone(),
            party_id: command.party_id.clone(),
            status: "queued".into(),
            priority: command.priority,
            match_attributes: command.match_attributes.clone(),
            idempotency_key: command.idempotency_key.clone(),
            queued_at: timestamp,
            matched_at: None,
            cancelled_at: None,
            expires_at: command.expires_at.clone(),
            version: 0,
        };
        tickets.push(StoredTicket {
            tenant_id: tenant_id.into(),
            item: item.clone(),
        });
        Ok(item)
    }

    async fn get_ticket(
        &self,
        tenant_id: &str,
        ticket_id: &str,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        let tickets = self.tickets.read().map_err(lock_error)?;
        tickets
            .iter()
            .find(|stored| {
                stored.tenant_id == tenant_id
                    && (stored.item.id == ticket_id || stored.item.ticket_code == ticket_id)
            })
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GameMatchmakingError::not_found("match ticket not found"))
    }

    async fn cancel_ticket(
        &self,
        tenant_id: &str,
        command: &CancelMatchTicketCommand,
    ) -> GameMatchmakingResult<MatchTicketItem> {
        let mut tickets = self.tickets.write().map_err(lock_error)?;
        let stored = tickets
            .iter_mut()
            .find(|stored| {
                stored.tenant_id == tenant_id
                    && stored.item.user_id == command.user_id
                    && (stored.item.id == command.ticket_id
                        || stored.item.ticket_code == command.ticket_id)
            })
            .ok_or_else(|| GameMatchmakingError::not_found("match ticket not found"))?;
        if stored.item.status != "queued" {
            return Err(GameMatchmakingError::conflict(
                "only queued match tickets can be cancelled",
            ));
        }
        stored.item.status = "cancelled".into();
        stored.item.cancelled_at = Some(now().to_rfc3339());
        stored.item.version += 1;
        Ok(stored.item.clone())
    }

    async fn list_tickets(
        &self,
        tenant_id: &str,
        query: &MatchTicketQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        let tickets = self.tickets.read().map_err(lock_error)?;
        let mut filtered: Vec<MatchTicketItem> = tickets
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id)
            .filter(|stored| matches_ticket_query(&stored.item, query))
            .map(|stored| stored.item.clone())
            .collect();
        filtered.sort_by(|left, right| right.queued_at.cmp(&left.queued_at));
        Ok(page_from_items(
            filtered,
            query.page.unwrap_or(1),
            query.limit(),
            query.offset(),
        ))
    }

    async fn list_queue(
        &self,
        tenant_id: &str,
        query: &MatchmakingQueueQuery,
    ) -> GameMatchmakingResult<MatchTicketPage> {
        let tickets = self.tickets.read().map_err(lock_error)?;
        let mut filtered: Vec<MatchTicketItem> = tickets
            .iter()
            .filter(|stored| stored.tenant_id == tenant_id)
            .filter(|stored| stored.item.status == "queued")
            .filter(|stored| stored.item.game_id == query.game_id)
            .filter(|stored| {
                query
                    .mode_id
                    .as_deref()
                    .filter(|value| !is_blank(Some(value)))
                    .is_none_or(|mode_id| stored.item.mode_id.as_deref() == Some(mode_id))
            })
            .map(|stored| stored.item.clone())
            .collect();
        filtered.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.queued_at.cmp(&right.queued_at))
        });
        Ok(page_from_items(
            filtered,
            query.page.unwrap_or(1),
            query.limit(),
            query.offset(),
        ))
    }
}

fn matches_ticket_query(item: &MatchTicketItem, query: &MatchTicketQuery) -> bool {
    optional_eq(query.game_id.as_deref(), &item.game_id)
        && optional_eq(
            query.mode_id.as_deref(),
            item.mode_id.as_deref().unwrap_or_default(),
        )
        && optional_eq(query.status.as_deref(), &item.status)
        && optional_eq(query.user_id.as_deref(), &item.user_id)
}

fn optional_eq(expected: Option<&str>, actual: &str) -> bool {
    expected
        .filter(|value| !is_blank(Some(value)))
        .is_none_or(|value| value == actual)
}

fn page_from_items(
    items: Vec<MatchTicketItem>,
    page: u32,
    page_size: u32,
    offset: u32,
) -> MatchTicketPage {
    let total = items.len() as u64;
    let page_items = items
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();
    MatchTicketPage {
        items: page_items,
        total,
        page,
        page_size,
    }
}

fn ensure_idempotent_replay(
    existing: &MatchTicketItem,
    command: &CreateMatchTicketCommand,
) -> GameMatchmakingResult<()> {
    let same_payload = existing.game_id == command.game_id
        && existing.mode_id == command.mode_id
        && existing.ruleset_id == command.ruleset_id
        && existing.user_id == command.user_id
        && existing.party_id == command.party_id
        && existing.priority == command.priority
        && existing.match_attributes == command.match_attributes
        && existing.expires_at == command.expires_at;
    if !same_payload {
        return Err(GameMatchmakingError::conflict(
            "idempotency_key already belongs to a different match ticket payload",
        ));
    }
    Ok(())
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GameMatchmakingError {
    GameMatchmakingError::invalid("matchmaking repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use sdkwork_game_matchmaking_service::{
        CancelMatchTicketCommand, CreateMatchTicketCommand, GameMatchmakingRepository,
    };
    use serde_json::json;

    use super::InMemoryGameMatchmakingRepository;

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

    #[tokio::test]
    async fn create_ticket_is_idempotent() {
        let repository = InMemoryGameMatchmakingRepository::default();
        let command = command("idem-memory-1");

        let first = repository.create_ticket("100001", &command).await.unwrap();
        let replay = repository.create_ticket("100001", &command).await.unwrap();

        assert_eq!(first.id, replay.id);
        assert_eq!("queued", replay.status);
    }

    #[tokio::test]
    async fn conflicting_idempotency_payload_fails() {
        let repository = InMemoryGameMatchmakingRepository::default();
        let command = command("idem-memory-conflict");
        repository.create_ticket("100001", &command).await.unwrap();

        let mut conflict = command.clone();
        conflict.user_id = "user-2".into();

        let error = repository
            .create_ticket("100001", &conflict)
            .await
            .unwrap_err();
        assert_eq!("conflict", error.code());
    }

    #[tokio::test]
    async fn cancel_ticket_updates_queued_ticket() {
        let repository = InMemoryGameMatchmakingRepository::default();
        let ticket = repository
            .create_ticket("100001", &command("idem-memory-cancel"))
            .await
            .unwrap();

        let cancelled = repository
            .cancel_ticket(
                "100001",
                &CancelMatchTicketCommand {
                    ticket_id: ticket.id,
                    user_id: "user-1".into(),
                    reason: "player_cancelled".into(),
                },
            )
            .await
            .unwrap();

        assert_eq!("cancelled", cancelled.status);
        assert!(cancelled.cancelled_at.is_some());
    }
}
