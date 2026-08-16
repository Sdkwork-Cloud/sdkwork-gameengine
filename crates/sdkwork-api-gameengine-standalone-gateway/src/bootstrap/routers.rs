use std::sync::Arc;

use axum::Router;
use sdkwork_api_gameengine_assembly::{ApiAssembly, ApiAssemblyRuntime};
use sdkwork_iam_web_adapter::{
    build_web_framework_builder, iam_web_request_context_resolver_from_database_pool_for_audiences,
    iam_web_request_context_resolver_from_env, IamAuditEmitter, IamSecurityEventEmitter,
};
use sdkwork_web_bootstrap::{infra_public_path_prefixes, ComposedApiAssembly};
use sdkwork_web_core::WebRequestContextResolver;

const APPLICATION_ID: &str = "sdkwork-gameengine";
const APPLICATION_AUDIENCES: &[&str] = &["games", "sdkwork-games", APPLICATION_ID];

pub async fn build_router(runtime: ApiAssemblyRuntime) -> Result<Router, String> {
    let environment = std::env::var("SDKWORK_ENVIRONMENT")
        .or_else(|_| std::env::var("SDKWORK_GAMEENGINE_ENVIRONMENT"))
        .or_else(|_| std::env::var("SDKWORK_GAMES_ENVIRONMENT"))
        .unwrap_or_else(|_| "development".to_owned());
    let production = matches!(
        environment.trim().to_ascii_lowercase().as_str(),
        "prod" | "production"
    );
    let resolver = if production {
        iam_web_request_context_resolver_from_database_pool_for_audiences(
            runtime.database_pool.clone(),
            APPLICATION_AUDIENCES,
        )
        .await?
    } else {
        iam_web_request_context_resolver_from_env().await
    };
    let assembly = runtime.contribution;
    let mut framework = build_web_framework_builder(
        resolver,
        assembly.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    if production {
        let postgres_pool = runtime
            .database_pool
            .as_postgres()
            .cloned()
            .ok_or_else(|| "production GameEngine gateway requires PostgreSQL".to_owned())?;
        framework = framework
            .audit_emitter(Arc::new(IamAuditEmitter::new(
                postgres_pool.clone(),
                APPLICATION_ID,
                environment.clone(),
            )))
            .security_event_emitter(Arc::new(IamSecurityEventEmitter::new(
                postgres_pool,
                environment,
            )));
    }
    Ok(
        ComposedApiAssembly::try_compose("SDKWork Game Engine API", vec![assembly])?
            .into_hosted(framework)
            .router,
    )
}

pub fn build_router_with_resolver<R>(assembly: ApiAssembly, resolver: R) -> Result<Router, String>
where
    R: WebRequestContextResolver + Clone + Send + Sync + 'static,
{
    let framework = build_web_framework_builder(
        resolver,
        assembly.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    Ok(
        ComposedApiAssembly::try_compose("SDKWork Game Engine API", vec![assembly])?
            .into_hosted(framework)
            .router,
    )
}
