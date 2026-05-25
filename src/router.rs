use axum::{Router, routing::{get, delete}};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/login", get(handlers::auth::login))
        .route("/callback", get(handlers::auth::callback))
        .route("/logout", delete(handlers::auth::logout));

    let api_routes = Router::new()
        .route("/profile", get(handlers::profile::profile))
        .route("/body", get(handlers::profile::body))
        .route("/cycles", get(handlers::cycles::list_cycles))
        .route("/cycles/{id}", get(handlers::cycles::get_cycle))
        .route("/cycles/{id}/recovery", get(handlers::recovery::get_recovery))
        .route("/cycles/{id}/sleep", get(handlers::sleep::get_cycle_sleep))
        .route("/sleep", get(handlers::sleep::list_sleep))
        .route("/sleep/{id}", get(handlers::sleep::get_sleep))
        .route("/workouts", get(handlers::workouts::list_workouts))
        .route("/workouts/{id}", get(handlers::workouts::get_workout));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
