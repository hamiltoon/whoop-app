use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;
use crate::whoop::auth as whoop_auth;
use crate::whoop::client::AppError;

use super::extract_session;

pub async fn login(State(state): State<AppState>) -> Redirect {
    let url = whoop_auth::build_auth_url(&state.config);
    Redirect::temporary(&url)
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}

pub async fn callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let token_pair = whoop_auth::exchange_code(
        &state.http_client,
        &state.config,
        &query.code,
    )
    .await
    .map_err(|e| AppError::InternalError(e))?;

    let session_id = Uuid::new_v4();

    {
        let mut tokens = state.tokens.write().await;
        tokens.insert(session_id, token_pair);
    }

    let cookie = format!("whoop_session={session_id}; Path=/; HttpOnly; SameSite=Lax");

    let mut response = Redirect::temporary("/").into_response();
    response.headers_mut().insert(
        "set-cookie",
        cookie.parse().unwrap(),
    );

    Ok(response)
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let session_id = extract_session(&headers)?;

    {
        let mut tokens = state.tokens.write().await;
        tokens.remove(&session_id);
    }

    let cookie = "whoop_session=; Path=/; HttpOnly; Max-Age=0";

    Ok((
        StatusCode::OK,
        [("set-cookie", cookie)],
        axum::Json(serde_json::json!({"message": "Logged out"})),
    ))
}
