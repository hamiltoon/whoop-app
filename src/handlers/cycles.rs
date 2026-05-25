use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;

use crate::state::AppState;
use crate::whoop::client::{ApiResult, AppError, WhoopClient};
use crate::whoop::models::PaginationParams;

use super::extract_session;

pub async fn list_cycles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let session_id = extract_session(&headers)?;
    let client = WhoopClient::new(&state, session_id);

    match client.get("/v2/cycle", Some(&params)).await? {
        ApiResult::Success(body) => Ok(axum::Json(body).into_response()),
        ApiResult::Error { status, body } => Err(AppError::WhoopApiError { status, body }),
    }
}

pub async fn get_cycle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let session_id = extract_session(&headers)?;
    let client = WhoopClient::new(&state, session_id);
    let path = format!("/v2/cycle/{id}");

    match client.get(&path, None).await? {
        ApiResult::Success(body) => Ok(axum::Json(body).into_response()),
        ApiResult::Error { status, body } => Err(AppError::WhoopApiError { status, body }),
    }
}
