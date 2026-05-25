use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;

use crate::state::AppState;
use crate::whoop::client::{ApiResult, AppError, WhoopClient};

use super::extract_session;

pub async fn profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let session_id = extract_session(&headers)?;
    let client = WhoopClient::new(&state, session_id);

    match client.get("/v2/user/profile/basic", None).await? {
        ApiResult::Success(body) => Ok(axum::Json(body).into_response()),
        ApiResult::Error { status, body } => Err(AppError::WhoopApiError { status, body }),
    }
}

pub async fn body(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let session_id = extract_session(&headers)?;
    let client = WhoopClient::new(&state, session_id);

    match client.get("/v2/user/measurement/body", None).await? {
        ApiResult::Success(body) => Ok(axum::Json(body).into_response()),
        ApiResult::Error { status, body } => Err(AppError::WhoopApiError { status, body }),
    }
}
