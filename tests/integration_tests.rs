use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use uuid::Uuid;

use whoop_app::config::Config;
use whoop_app::router;
use whoop_app::state::AppState;
use whoop_app::whoop::models::TokenPair;

fn test_config() -> Config {
    Config {
        client_id: "test_id".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:3000/auth/callback".to_string(),
        port: 3000,
    }
}

fn test_state() -> AppState {
    AppState::new(test_config())
}

fn app() -> axum::Router {
    router::build(test_state())
}

async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// --- Auth handler tests ---

#[tokio::test]
async fn login_redirects_to_whoop_oauth() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = resp.headers().get("location").unwrap().to_str().unwrap();
    assert!(location.contains("oauth2/auth"));
    assert!(location.contains("client_id=test_id"));
}

#[tokio::test]
async fn logout_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_with_valid_session_clears_cookie() {
    let state = test_state();
    let session_id = Uuid::new_v4();

    // Insert a session
    {
        let mut tokens = state.tokens.write().await;
        tokens.insert(
            session_id,
            TokenPair {
                access_token: "acc".to_string(),
                refresh_token: "ref".to_string(),
                expires_at: chrono::Utc::now(),
            },
        );
    }

    let app = router::build(state.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/auth/logout")
                .header("cookie", format!("whoop_session={session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let set_cookie = resp.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(set_cookie.contains("Max-Age=0"));

    // Verify token removed from store
    let tokens = state.tokens.read().await;
    assert!(!tokens.contains_key(&session_id));
}

// --- API endpoint auth-guard tests ---

#[tokio::test]
async fn api_profile_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("Unauthorized"));
}

#[tokio::test]
async fn api_body_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/body")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_cycles_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/cycles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_cycles_by_id_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/cycles/123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_recovery_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/cycles/123/recovery")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_sleep_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/sleep")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_sleep_by_id_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/sleep/456")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_workouts_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/workouts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_workouts_by_id_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/workouts/789")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_cycle_sleep_without_session_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/cycles/123/sleep")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// --- Invalid session UUID tests ---

#[tokio::test]
async fn api_profile_with_invalid_session_uuid_returns_401() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/profile")
                .header("cookie", "whoop_session=not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// --- Valid session but no token in store (session_id not found) ---

#[tokio::test]
async fn api_profile_with_unknown_session_returns_401() {
    let fake_session = Uuid::new_v4();
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/profile")
                .header("cookie", format!("whoop_session={fake_session}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // The handler calls WhoopClient::get which checks the token store
    // and returns Unauthorized when the session is not found
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// --- Non-existent routes ---

#[tokio::test]
async fn unknown_route_returns_404() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// --- Auth callback without code param ---

#[tokio::test]
async fn auth_callback_without_code_returns_400() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/auth/callback")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing required query parameter "code" -> 400 from axum's Query extractor
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
