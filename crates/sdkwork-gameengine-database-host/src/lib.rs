use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::claw_database::{
    build_postgres_database_url, postgres_url_with_search_path,
};
use sdkwork_database_config::{DatabaseConfig, DatabaseEngine, PgSslMode, PostgresConfig};
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};

pub struct GamesDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl GamesDatabaseHost {
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

pub async fn bootstrap_games_database(pool: DatabasePool) -> Result<GamesDatabaseHost, String> {
    let app_root = resolve_app_root();
    let module = Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load games database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read games database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("GAMES", &manifest);
    let orchestrator =
        LifecycleOrchestrator::new(pool.clone(), module.clone()).with_applied_by("sdkwork-games");

    orchestrator
        .init()
        .await
        .map_err(|error| format!("games database init failed: {error}"))?;

    if options.auto_migrate {
        orchestrator
            .migrate()
            .await
            .map_err(|error| format!("games database migrate failed: {error}"))?;
    }

    Ok(GamesDatabaseHost { pool, module })
}

pub async fn bootstrap_games_database_from_env() -> Result<GamesDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = resolve_games_database_config_from_env()?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create games database pool failed: {error}"))?;
    bootstrap_games_database(pool).await
}

fn resolve_games_database_config_from_env() -> Result<DatabaseConfig, String> {
    if let Some(url) = env_string("SDKWORK_GAMES_DATABASE_URL") {
        let mut config = DatabaseConfig::from_env("GAMES")
            .map_err(|error| format!("read games database config failed: {error}"))?;
        config.url = url;
        return Ok(config);
    }

    match env_string("SDKWORK_GAMES_DATABASE_ENGINE").as_deref() {
        Some("postgres") | Some("postgresql") => resolve_structured_postgres_config(),
        Some("sqlite") => DatabaseConfig::from_env("GAMES")
            .map_err(|error| format!("read games database config failed: {error}")),
        Some(other) => Err(format!(
            "unsupported SDKWORK_GAMES_DATABASE_ENGINE: {other}; expected postgresql or sqlite"
        )),
        None if is_production_environment() => Err(
            "SDKWORK_GAMES_DATABASE_ENGINE or SDKWORK_GAMES_DATABASE_URL is required for production"
                .to_string(),
        ),
        None => DatabaseConfig::from_env("GAMES")
            .map_err(|error| format!("read games database config failed: {error}")),
    }
}

fn resolve_structured_postgres_config() -> Result<DatabaseConfig, String> {
    let host = required_env("SDKWORK_GAMES_DATABASE_HOST")?;
    let database = required_env("SDKWORK_GAMES_DATABASE_NAME")?;
    let username = required_env("SDKWORK_GAMES_DATABASE_USERNAME")?;
    let ssl_mode = env_string("SDKWORK_GAMES_DATABASE_SSL_MODE").unwrap_or_else(|| "prefer".into());
    let port = env_string("SDKWORK_GAMES_DATABASE_PORT");
    let password = resolve_database_password()?;
    let database_url = build_postgres_database_url(
        &host,
        port.as_deref(),
        &database,
        &username,
        &password,
        Some(&ssl_mode),
    );
    let default_config = DatabaseConfig::default();
    Ok(DatabaseConfig {
        engine: DatabaseEngine::Postgres,
        url: postgres_url_with_search_path(&database_url, "SDKWORK_GAMES"),
        max_connections: env_u32(
            "SDKWORK_GAMES_DATABASE_MAX_CONNECTIONS",
            default_config.max_connections,
        )?,
        postgres: PostgresConfig {
            ssl_mode: parse_pg_ssl_mode(&ssl_mode),
            ..Default::default()
        },
        ..default_config
    })
}

fn resolve_database_password() -> Result<String, String> {
    if let Some(path) = env_string("SDKWORK_GAMES_DATABASE_PASSWORD_FILE") {
        return std::fs::read_to_string(&path)
            .map(|value| value.trim_end_matches(['\r', '\n']).to_string())
            .map_err(|error| format!("read SDKWORK_GAMES_DATABASE_PASSWORD_FILE failed: {error}"));
    }

    if is_production_environment() {
        return Err(
            "SDKWORK_GAMES_DATABASE_PASSWORD_FILE is required for production PostgreSQL config"
                .to_string(),
        );
    }

    required_env("SDKWORK_GAMES_DATABASE_PASSWORD")
}

fn is_production_environment() -> bool {
    matches!(
        env_string("SDKWORK_GAMES_ENVIRONMENT").as_deref(),
        Some("production")
    )
}

fn required_env(key: &str) -> Result<String, String> {
    env_string(key).ok_or_else(|| format!("{key} is required"))
}

fn env_string(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_u32(key: &str, default: u32) -> Result<u32, String> {
    match env_string(key) {
        Some(value) => value
            .parse::<u32>()
            .map_err(|_| format!("{key} must be an unsigned integer")),
        None => Ok(default),
    }
}

fn parse_pg_ssl_mode(value: &str) -> PgSslMode {
    match value.trim().to_ascii_lowercase().as_str() {
        "disable" => PgSslMode::Disable,
        "allow" => PgSslMode::Allow,
        "prefer" => PgSslMode::Prefer,
        "require" => PgSslMode::Require,
        "verify-ca" | "verify_ca" => PgSslMode::VerifyCa,
        "verify-full" | "verify_full" => PgSslMode::VerifyFull,
        _ => PgSslMode::Prefer,
    }
}

fn resolve_app_root() -> PathBuf {
    std::env::var("SDKWORK_GAMES_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_database_config::DatabaseEngine;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct EnvGuard {
        previous: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn set(values: &[(&str, Option<&str>)]) -> Self {
            let previous = values
                .iter()
                .map(|(key, _)| ((*key).to_string(), env::var(*key).ok()))
                .collect::<Vec<_>>();
            for (key, value) in values {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
            Self { previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.previous {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }

    fn temp_secret_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        env::temp_dir().join(format!("sdkwork-games-db-secret-{nanos}.txt"))
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn resolves_structured_games_postgres_database_config() {
        let _lock = env_lock().lock().expect("env lock");
        let password_path = temp_secret_path();
        fs::write(&password_path, "secret value\n").expect("write secret");
        let password_path_string = password_path.to_string_lossy().to_string();
        let _guard = EnvGuard::set(&[
            ("SDKWORK_GAMES_DATABASE_URL", None),
            ("SDKWORK_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("SDKWORK_CLAW_DATABASE_URL", None),
            ("SDKWORK_CLAW_DATABASE_ENGINE", None),
            ("SDKWORK_GAMES_DATABASE_ENGINE", Some("postgresql")),
            ("SDKWORK_GAMES_DATABASE_HOST", Some("db.internal")),
            ("SDKWORK_GAMES_DATABASE_PORT", Some("5433")),
            ("SDKWORK_GAMES_DATABASE_NAME", Some("sdkwork_games_prod")),
            ("SDKWORK_GAMES_DATABASE_SCHEMA", Some("sdkwork_games_prod")),
            ("SDKWORK_GAMES_DATABASE_USERNAME", Some("sdkwork_games")),
            (
                "SDKWORK_GAMES_DATABASE_PASSWORD_FILE",
                Some(password_path_string.as_str()),
            ),
            ("SDKWORK_GAMES_DATABASE_PASSWORD", None),
            ("SDKWORK_GAMES_DATABASE_SSL_MODE", Some("require")),
            ("SDKWORK_GAMES_DATABASE_MAX_CONNECTIONS", Some("24")),
        ]);

        let config = resolve_games_database_config_from_env().expect("database config");

        assert_eq!(config.engine, DatabaseEngine::Postgres);
        assert_eq!(config.max_connections, 24);
        assert!(config.url.starts_with(
            "postgresql://sdkwork_games:secret%20value@db.internal:5433/sdkwork_games_prod"
        ));
        assert!(config.url.contains("sslmode=require"));
        assert!(config
            .url
            .contains("options=-c%20search_path%3Dsdkwork_games_prod%2Cpublic"));

        let _ = fs::remove_file(password_path);
    }

    #[test]
    fn production_structured_postgres_requires_password_file() {
        let _lock = env_lock().lock().expect("env lock");
        let _guard = EnvGuard::set(&[
            ("SDKWORK_GAMES_DATABASE_URL", None),
            ("SDKWORK_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("SDKWORK_CLAW_DATABASE_URL", None),
            ("SDKWORK_CLAW_DATABASE_ENGINE", None),
            ("SDKWORK_GAMES_ENVIRONMENT", Some("production")),
            ("SDKWORK_GAMES_DATABASE_ENGINE", Some("postgresql")),
            ("SDKWORK_GAMES_DATABASE_HOST", Some("db.internal")),
            ("SDKWORK_GAMES_DATABASE_PORT", Some("5432")),
            ("SDKWORK_GAMES_DATABASE_NAME", Some("sdkwork_games_prod")),
            ("SDKWORK_GAMES_DATABASE_USERNAME", Some("sdkwork_games")),
            ("SDKWORK_GAMES_DATABASE_PASSWORD_FILE", None),
            ("SDKWORK_GAMES_DATABASE_PASSWORD", Some("inline-secret")),
            ("SDKWORK_GAMES_DATABASE_SSL_MODE", Some("require")),
        ]);

        let error = resolve_games_database_config_from_env().expect_err("missing password file");

        assert!(error.contains("SDKWORK_GAMES_DATABASE_PASSWORD_FILE"));
    }

    #[test]
    fn production_requires_explicit_database_config() {
        let _lock = env_lock().lock().expect("env lock");
        let _guard = EnvGuard::set(&[
            ("SDKWORK_GAMES_DATABASE_URL", None),
            ("SDKWORK_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("SDKWORK_CLAW_DATABASE_URL", None),
            ("SDKWORK_CLAW_DATABASE_ENGINE", None),
            ("SDKWORK_GAMES_ENVIRONMENT", Some("production")),
            ("SDKWORK_GAMES_DATABASE_ENGINE", None),
        ]);

        let error =
            resolve_games_database_config_from_env().expect_err("production database config");

        assert!(error.contains("SDKWORK_GAMES_DATABASE_ENGINE"));
    }
}
