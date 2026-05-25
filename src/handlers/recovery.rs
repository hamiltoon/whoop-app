use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;

use crate::state::AppState;
use crate::whoop::client::{ApiResult, AppError, WhoopClient};

use super::extract_session;

pub async fn get_recovery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let session_id = extract_session(&headers)?;
    let client = WhoopClient::new(&state, session_id);
    let path = format!("/v2/cycle/{id}/recovery");

    match client.get(&path, None).await? {
        ApiResult::Success(body) => Ok(axum::Json(body).into_response()),
        ApiResult::Error { status, body } => Err(AppError::WhoopApiError { status, body }),
    }
}
