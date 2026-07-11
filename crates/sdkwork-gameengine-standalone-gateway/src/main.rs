use sdkwork_gameengine_standalone_gateway::{build_gateway_services, build_router};

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

    let services = build_gateway_services()
        .await
        .expect("gameengine gateway services bootstrap failed");
    let app = build_router(services.catalog, services.leaderboard, services.room).await;
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("bind gameengine standalone-gateway listener failed");
    tracing::info!("sdkwork-gameengine-standalone-gateway listening on {bind_address}");
    axum::serve(listener, app)
        .await
        .expect("serve gameengine standalone-gateway failed");
}
