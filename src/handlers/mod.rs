pub mod auth;
pub mod cycles;
pub mod profile;
pub mod recovery;
pub mod sleep;
pub mod workouts;

use axum::http::HeaderMap;
use uuid::Uuid;

use crate::state::SessionId;
use crate::whoop::client::AppError;

pub fn extract_session(headers: &HeaderMap) -> Result<SessionId, AppError> {
    let cookie_header = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix("whoop_session=") {
            let id = Uuid::parse_str(value).map_err(|_| AppError::Unauthorized)?;
            return Ok(id);
        }
    }

    Err(AppError::Unauthorized)
}
