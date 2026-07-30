use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::workspace_database::workspace_database_env_is_configured;
use sdkwork_database_config::DatabaseConfig;
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
    if is_production_environment() {
        if !workspace_database_env_is_configured() {
            return Err(
                "SDKWORK_DATABASE_ENGINE or SDKWORK_DATABASE_URL is required for production"
                    .to_string(),
            );
        }
        let structured_postgres = matches!(
            env_string("SDKWORK_DATABASE_ENGINE")
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("postgres") | Some("postgresql")
        ) && env_string("SDKWORK_DATABASE_URL").is_none();
        if structured_postgres && env_string("SDKWORK_DATABASE_PASSWORD_FILE").is_none() {
            return Err(
                "SDKWORK_DATABASE_PASSWORD_FILE is required for production PostgreSQL config"
                    .to_string(),
            );
        }
    }

    DatabaseConfig::from_env("GAMES")
        .map_err(|error| format!("read games database config failed: {error}"))
}

fn is_production_environment() -> bool {
    matches!(
        env_string("SDKWORK_GAMES_ENVIRONMENT").as_deref(),
        Some("production")
    )
}

fn env_string(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
            ("SDKWORK_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("SDKWORK_GAMES_ENVIRONMENT", None),
            ("SDKWORK_DATABASE_ENGINE", Some("postgresql")),
            ("SDKWORK_DATABASE_HOST", Some("db.internal")),
            ("SDKWORK_DATABASE_PORT", Some("5433")),
            ("SDKWORK_DATABASE_NAME", Some("sdkwork_ai_prod")),
            ("SDKWORK_DATABASE_SCHEMA", Some("sdkwork_ai_prod")),
            ("SDKWORK_DATABASE_USERNAME", Some("sdkwork_ai_prod")),
            (
                "SDKWORK_DATABASE_PASSWORD_FILE",
                Some(password_path_string.as_str()),
            ),
            ("SDKWORK_DATABASE_PASSWORD", None),
            ("SDKWORK_DATABASE_SSL_MODE", Some("require")),
            ("SDKWORK_DATABASE_MAX_CONNECTIONS", Some("24")),
        ]);

        let config = resolve_games_database_config_from_env().expect("database config");

        assert_eq!(config.engine, DatabaseEngine::Postgres);
        assert_eq!(config.max_connections, 24);
        assert!(config.url.starts_with(
            "postgresql://sdkwork_ai_prod:secret%20value@db.internal:5433/sdkwork_ai_prod"
        ));
        assert!(config.url.contains("sslmode=require"));
        assert!(config
            .url
            .contains("search_path%3Dsdkwork_ai_prod%2Cpublic"));

        let _ = fs::remove_file(password_path);
    }

    #[test]
    fn production_structured_postgres_requires_password_file() {
        let _lock = env_lock().lock().expect("env lock");
        let _guard = EnvGuard::set(&[
            ("SDKWORK_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("SDKWORK_GAMES_ENVIRONMENT", Some("production")),
            ("SDKWORK_DATABASE_ENGINE", Some("postgresql")),
            ("SDKWORK_DATABASE_HOST", Some("db.internal")),
            ("SDKWORK_DATABASE_PORT", Some("5432")),
            ("SDKWORK_DATABASE_NAME", Some("sdkwork_ai_prod")),
            ("SDKWORK_DATABASE_SCHEMA", Some("sdkwork_ai_prod")),
            ("SDKWORK_DATABASE_USERNAME", Some("sdkwork_ai_prod")),
            ("SDKWORK_DATABASE_PASSWORD_FILE", None),
            ("SDKWORK_DATABASE_PASSWORD", Some("inline-secret")),
            ("SDKWORK_DATABASE_SSL_MODE", Some("require")),
        ]);

        let error = resolve_games_database_config_from_env().expect_err("missing password file");

        assert!(error.contains("SDKWORK_DATABASE_PASSWORD_FILE"));
    }

    #[test]
    fn production_requires_explicit_database_config() {
        let _lock = env_lock().lock().expect("env lock");
        let _guard = EnvGuard::set(&[
            ("SDKWORK_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("SDKWORK_GAMES_ENVIRONMENT", Some("production")),
            ("SDKWORK_DATABASE_ENGINE", None),
            ("SDKWORK_DATABASE_HOST", None),
            ("SDKWORK_DATABASE_PORT", None),
            ("SDKWORK_DATABASE_NAME", None),
            ("SDKWORK_DATABASE_SCHEMA", None),
            ("SDKWORK_DATABASE_USERNAME", None),
            ("SDKWORK_DATABASE_PASSWORD", None),
            ("SDKWORK_DATABASE_PASSWORD_FILE", None),
            ("SDKWORK_DATABASE_FILE", None),
        ]);

        let error =
            resolve_games_database_config_from_env().expect_err("production database config");

        assert!(error.contains("SDKWORK_DATABASE_ENGINE"));
    }
}
