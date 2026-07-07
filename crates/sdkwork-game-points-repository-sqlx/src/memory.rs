use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sdkwork_game_points_service::{
    AppendPointLedgerCommand, GamePointBalance, GamePointError, GamePointLedgerEntry,
    GamePointRepository, GamePointResult,
};
use sdkwork_utils_rust::datetime::now;
use sdkwork_utils_rust::id::uuid;

#[derive(Clone, Default)]
pub struct InMemoryGamePointRepository {
    store: Arc<RwLock<PointStore>>,
}

#[derive(Default)]
struct PointStore {
    ledgers: Vec<StoredLedger>,
    balances: Vec<StoredBalance>,
}

#[derive(Clone)]
struct StoredLedger {
    tenant_id: String,
    item: GamePointLedgerEntry,
}

#[derive(Clone)]
struct StoredBalance {
    tenant_id: String,
    item: GamePointBalance,
}

#[async_trait]
impl GamePointRepository for InMemoryGamePointRepository {
    async fn append_ledger(
        &self,
        tenant_id: &str,
        command: &AppendPointLedgerCommand,
    ) -> GamePointResult<GamePointLedgerEntry> {
        let mut store = self.store.write().map_err(lock_error)?;

        if let Some(existing) = store.ledgers.iter().find(|stored| {
            stored.tenant_id == tenant_id && stored.item.idempotency_key == command.idempotency_key
        }) {
            ensure_idempotent_replay(&existing.item, command)?;
            return Ok(existing.item.clone());
        }

        let signed_delta = signed_delta(command);
        let balance_index = store.balances.iter().position(|stored| {
            stored.tenant_id == tenant_id
                && stored.item.ledger_account_id == command.ledger_account_id
        });
        let points_after = balance_index
            .map(|index| store.balances[index].item.points)
            .unwrap_or_default()
            + signed_delta;

        let timestamp = now().to_rfc3339();
        let entry = GamePointLedgerEntry {
            id: uuid(),
            ledger_account_id: command.ledger_account_id.clone(),
            game_id: command.game_id.clone(),
            mode_id: command.mode_id.clone(),
            season_id: command.season_id.clone(),
            user_id: command.user_id.clone(),
            direction: command.direction.clone(),
            points_delta: command.points_delta,
            points_after,
            source_event_id: command.source_event_id.clone(),
            reason_code: command.reason_code.clone(),
            idempotency_key: command.idempotency_key.clone(),
            created_at: timestamp.clone(),
            version: 0,
        };

        if let Some(index) = balance_index {
            let balance = &mut store.balances[index].item;
            balance.points = points_after;
            balance.last_ledger_id = Some(entry.id.clone());
            balance.updated_at = timestamp;
            balance.version += 1;
        } else {
            store.balances.push(StoredBalance {
                tenant_id: tenant_id.into(),
                item: GamePointBalance {
                    id: uuid(),
                    ledger_account_id: command.ledger_account_id.clone(),
                    game_id: command.game_id.clone(),
                    mode_id: command.mode_id.clone(),
                    season_id: command.season_id.clone(),
                    user_id: command.user_id.clone(),
                    points: points_after,
                    last_ledger_id: Some(entry.id.clone()),
                    updated_at: timestamp,
                    version: 0,
                },
            });
        }

        store.ledgers.push(StoredLedger {
            tenant_id: tenant_id.into(),
            item: entry.clone(),
        });

        Ok(entry)
    }

    async fn get_balance(
        &self,
        tenant_id: &str,
        ledger_account_id: &str,
    ) -> GamePointResult<GamePointBalance> {
        let store = self.store.read().map_err(lock_error)?;
        store
            .balances
            .iter()
            .find(|stored| {
                stored.tenant_id == tenant_id && stored.item.ledger_account_id == ledger_account_id
            })
            .map(|stored| stored.item.clone())
            .ok_or_else(|| GamePointError::not_found("point balance not found"))
    }
}

fn signed_delta(command: &AppendPointLedgerCommand) -> i64 {
    if command.direction == "debit" {
        -command.points_delta
    } else {
        command.points_delta
    }
}

fn ensure_idempotent_replay(
    existing: &GamePointLedgerEntry,
    command: &AppendPointLedgerCommand,
) -> GamePointResult<()> {
    let same_payload = existing.ledger_account_id == command.ledger_account_id
        && existing.game_id == command.game_id
        && existing.mode_id == command.mode_id
        && existing.season_id == command.season_id
        && existing.user_id == command.user_id
        && existing.direction == command.direction
        && existing.points_delta == command.points_delta
        && existing.source_event_id == command.source_event_id
        && existing.reason_code == command.reason_code;
    if !same_payload {
        return Err(GamePointError::conflict(
            "idempotency_key already belongs to a different point ledger payload",
        ));
    }
    Ok(())
}

fn lock_error<T>(_: std::sync::PoisonError<T>) -> GamePointError {
    GamePointError::invalid("point repository lock is poisoned")
}

#[cfg(test)]
mod tests {
    use sdkwork_game_points_service::{AppendPointLedgerCommand, GamePointRepository};

    use super::InMemoryGamePointRepository;

    fn win_points_command(idempotency_key: &str) -> AppendPointLedgerCommand {
        AppendPointLedgerCommand {
            ledger_account_id: "acct-game-1-user-1".into(),
            game_id: "game-xiangqi".into(),
            mode_id: Some("mode-ranked".into()),
            season_id: Some("season-2026".into()),
            user_id: "user-1".into(),
            direction: "credit".into(),
            points_delta: 30,
            source_event_id: "score-event-1".into(),
            reason_code: "match_win".into(),
            idempotency_key: idempotency_key.into(),
        }
    }

    #[tokio::test]
    async fn append_ledger_is_idempotent_and_updates_balance_once() {
        let repo = InMemoryGamePointRepository::default();
        let command = win_points_command("idem-match-1-user-1");

        let first = repo.append_ledger("100001", &command).await.unwrap();
        let second = repo.append_ledger("100001", &command).await.unwrap();
        let balance = repo
            .get_balance("100001", &command.ledger_account_id)
            .await
            .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(30, first.points_after);
        assert_eq!(30, second.points_after);
        assert_eq!(30, balance.points);
        assert_eq!(Some(first.id), balance.last_ledger_id);
    }

    #[tokio::test]
    async fn append_ledger_rejects_conflicting_idempotency_payload() {
        let repo = InMemoryGamePointRepository::default();
        let command = win_points_command("idem-conflict");
        repo.append_ledger("100001", &command).await.unwrap();

        let mut conflict = command.clone();
        conflict.points_delta = 99;

        let error = repo.append_ledger("100001", &conflict).await.unwrap_err();
        assert_eq!("conflict", error.code());
    }

    #[tokio::test]
    async fn balances_are_isolated_by_tenant() {
        let repo = InMemoryGamePointRepository::default();
        let command = win_points_command("idem-tenant");

        repo.append_ledger("100001", &command).await.unwrap();
        repo.append_ledger("200001", &command).await.unwrap();

        let first = repo
            .get_balance("100001", &command.ledger_account_id)
            .await
            .unwrap();
        let second = repo
            .get_balance("200001", &command.ledger_account_id)
            .await
            .unwrap();

        assert_ne!(first.id, second.id);
        assert_eq!(30, first.points);
        assert_eq!(30, second.points);
    }
}
