use sdkwork_gameengine_gateway_assembly::assemble_application_router;
use sdkwork_gameengine_standalone_gateway::build_router_from_business;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let bind_address = std::env::var("GAMES_API_BIND")
        .or_else(|_| std::env::var("SDKWORK_GAMES_APPLICATION_PUBLIC_INGRESS_BIND"))
        .unwrap_or_else(|_| "127.0.0.1:8095".to_owned());

    let assembly = assemble_application_router()
        .await
        .expect("gameengine gateway assembly failed");
    let app = build_router_from_business(assembly.router);
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("bind gameengine standalone-gateway listener failed");
    tracing::info!("sdkwork-gameengine-standalone-gateway listening on {bind_address}");
    axum::serve(listener, app)
        .await
        .expect("serve gameengine standalone-gateway failed");
}
