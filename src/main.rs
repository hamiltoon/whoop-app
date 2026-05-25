use tracing_subscriber::EnvFilter;

mod config;
mod handlers;
mod router;
mod state;
mod whoop;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env().expect("Failed to load configuration");
    let port = config.port;
    let state = state::AppState::new(config);
    let app = router::build(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind to port");

    tracing::info!("Server listening on port {port}");
    axum::serve(listener, app).await.expect("Server error");
}
