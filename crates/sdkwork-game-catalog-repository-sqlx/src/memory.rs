use async_trait::async_trait;
use sdkwork_game_catalog_service::{
    GameCatalogItem, GameCatalogPage, GameCatalogQuery, GameCatalogRepository, GameError,
    GameResult,
};
use sdkwork_utils_rust::string::is_blank;

#[derive(Clone, Default)]
pub struct InMemoryGameCatalogRepository {
    items: Vec<GameCatalogItem>,
}

impl InMemoryGameCatalogRepository {
    pub fn with_seed(items: Vec<GameCatalogItem>) -> Self {
        Self { items }
    }
}

fn matches_query(item: &GameCatalogItem, query: &GameCatalogQuery) -> bool {
    if let Some(status) = &query.status {
        if item.status != *status {
            return false;
        }
    }

    if let Some(genre) = query
        .genre
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
    {
        if item.genre.as_deref() != Some(genre) {
            return false;
        }
    }

    if let Some(q) = query.q.as_deref().filter(|value| !is_blank(Some(value))) {
        let needle = q.trim().to_lowercase();
        let haystacks = [
            item.title.to_lowercase(),
            item.game_code.to_lowercase(),
            item.summary.as_deref().unwrap_or("").to_lowercase(),
        ];
        if !haystacks.iter().any(|value| value.contains(&needle)) {
            return false;
        }
    }

    true
}

fn sort_items(items: &mut [GameCatalogItem], sort: Option<&str>) {
    match sort {
        Some("title") => items.sort_by(|left, right| left.title.cmp(&right.title)),
        Some("newest") => items.sort_by(|left, right| right.game_code.cmp(&left.game_code)),
        _ => items.sort_by(|left, right| left.title.cmp(&right.title)),
    }
}

#[async_trait]
impl GameCatalogRepository for InMemoryGameCatalogRepository {
    async fn list_catalog(
        &self,
        _tenant_id: &str,
        query: &GameCatalogQuery,
    ) -> GameResult<GameCatalogPage> {
        let mut filtered: Vec<GameCatalogItem> = self
            .items
            .iter()
            .filter(|item| matches_query(item, query))
            .cloned()
            .collect();

        sort_items(&mut filtered, query.sort.as_deref());

        let total = filtered.len() as u64;
        let offset = query.offset() as usize;
        let limit = query.limit() as usize;
        let page_items = filtered.into_iter().skip(offset).take(limit).collect();

        Ok(GameCatalogPage {
            items: page_items,
            total,
            page: query.page.unwrap_or(1),
            page_size: query.limit(),
        })
    }

    async fn get_catalog_item(
        &self,
        _tenant_id: &str,
        game_id: &str,
    ) -> GameResult<GameCatalogItem> {
        self.items
            .iter()
            .find(|item| item.id == game_id || item.game_code == game_id)
            .cloned()
            .ok_or_else(|| GameError::not_found(format!("game {game_id} not found")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_game_catalog_service::GameCatalogQuery;

    fn sample_item(id: &str, genre: &str, title: &str) -> GameCatalogItem {
        GameCatalogItem {
            id: id.into(),
            game_code: id.into(),
            title: title.into(),
            summary: None,
            genre: Some(genre.into()),
            status: "published".into(),
        }
    }

    #[tokio::test]
    async fn list_catalog_paginates_items() {
        let repo = InMemoryGameCatalogRepository::with_seed(vec![sample_item(
            "g1",
            "puzzle",
            "Demo Game",
        )]);

        let page = repo
            .list_catalog("100001", &GameCatalogQuery::default())
            .await
            .expect("page");

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].title, "Demo Game");
    }

    #[tokio::test]
    async fn list_catalog_filters_by_genre_and_q() {
        let repo = InMemoryGameCatalogRepository::with_seed(vec![
            sample_item("g1", "chess", "Chinese Chess"),
            sample_item("g2", "quiz", "Quiz Arena"),
        ]);

        let page = repo
            .list_catalog(
                "100001",
                &GameCatalogQuery {
                    genre: Some("chess".into()),
                    q: Some("chinese".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("page");

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, "g1");
    }
}
