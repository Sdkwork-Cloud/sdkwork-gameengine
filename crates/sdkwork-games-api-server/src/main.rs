use sdkwork_games_api_server::{build_catalog_service, build_router};

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

    let catalog_service = build_catalog_service()
        .await
        .expect("games catalog service bootstrap failed");
    let app = build_router(catalog_service).await;
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("bind games api-server listener failed");
    tracing::info!("sdkwork-games-api-server listening on {bind_address}");
    axum::serve(listener, app)
        .await
        .expect("serve games api-server failed");
}
