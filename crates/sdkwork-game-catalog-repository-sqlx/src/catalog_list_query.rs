use sdkwork_game_catalog_service::GameCatalogQuery;
use sdkwork_utils_rust::string::is_blank;

const SELECT_COLUMNS: &str =
    "SELECT id, game_code, title, summary, genre, status FROM game_catalog";

pub struct CatalogListSql {
    pub select_sql: String,
    pub count_sql: String,
    pub bind_values: Vec<String>,
}

pub fn build_catalog_list_sql(query: &GameCatalogQuery, dialect: SqlDialect) -> CatalogListSql {
    let mut where_parts = vec![
        "tenant_id = ?".to_string(),
        "deleted_at IS NULL".to_string(),
    ];
    let mut bind_values = Vec::new();

    if let Some(status) = query
        .status
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
    {
        where_parts.push("status = ?".into());
        bind_values.push(status.to_string());
    }

    if let Some(genre) = query
        .genre
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
    {
        where_parts.push("genre = ?".into());
        bind_values.push(genre.to_string());
    }

    if let Some(q) = query.q.as_deref().filter(|value| !is_blank(Some(value))) {
        let pattern = format!("%{}%", escape_like(q.trim()));
        where_parts.push(
            "(LOWER(title) LIKE LOWER(?) OR LOWER(game_code) LIKE LOWER(?) OR LOWER(COALESCE(summary, '')) LIKE LOWER(?))"
                .into(),
        );
        bind_values.push(pattern.clone());
        bind_values.push(pattern.clone());
        bind_values.push(pattern);
    }

    let where_clause = where_parts.join(" AND ");
    let order_clause = order_by_clause(query.sort.as_deref());

    let select_sql =
        format!("{SELECT_COLUMNS} WHERE {where_clause} {order_clause} LIMIT ? OFFSET ?");
    let count_sql = format!("SELECT COUNT(*) FROM game_catalog WHERE {where_clause}");

    CatalogListSql {
        select_sql: dialect.rewrite_placeholders(&select_sql),
        count_sql: dialect.rewrite_placeholders(&count_sql),
        bind_values,
    }
}

fn order_by_clause(sort: Option<&str>) -> &'static str {
    match sort {
        Some("title") => "ORDER BY title ASC",
        Some("newest") => "ORDER BY created_at DESC, title ASC",
        _ => "ORDER BY sort_order ASC, title ASC",
    }
}

fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[derive(Clone, Copy)]
pub enum SqlDialect {
    Postgres,
    Sqlite,
}

impl SqlDialect {
    fn rewrite_placeholders(&self, sql: &str) -> String {
        match self {
            SqlDialect::Postgres => {
                let mut output = String::with_capacity(sql.len());
                let mut index = 1usize;
                for ch in sql.chars() {
                    if ch == '?' {
                        output.push('$');
                        output.push_str(&index.to_string());
                        index += 1;
                    } else {
                        output.push(ch);
                    }
                }
                output
            }
            SqlDialect::Sqlite => sql.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_postgres_placeholders() {
        let sql = build_catalog_list_sql(
            &GameCatalogQuery {
                status: Some("published".into()),
                genre: Some("chess".into()),
                ..Default::default()
            },
            SqlDialect::Postgres,
        );
        assert!(sql.select_sql.contains("$1"));
        assert!(sql.select_sql.contains("genre = $3"));
        assert_eq!(sql.bind_values.len(), 2);
    }

    #[test]
    fn q_filter_adds_three_bind_values() {
        let sql = build_catalog_list_sql(
            &GameCatalogQuery {
                q: Some("xiangqi".into()),
                ..Default::default()
            },
            SqlDialect::Sqlite,
        );
        assert_eq!(sql.bind_values.len(), 3);
        assert!(sql.select_sql.contains("LIKE LOWER(?)"));
    }
}
