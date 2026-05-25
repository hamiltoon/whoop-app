pub mod auth;
pub mod index;
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn extract_session_valid_cookie() {
        let mut headers = HeaderMap::new();
        let id = Uuid::new_v4();
        headers.insert("cookie", format!("whoop_session={id}").parse().unwrap());
        let result = extract_session(&headers).unwrap();
        assert_eq!(result, id);
    }

    #[test]
    fn extract_session_among_multiple_cookies() {
        let mut headers = HeaderMap::new();
        let id = Uuid::new_v4();
        headers.insert(
            "cookie",
            format!("other=abc; whoop_session={id}; foo=bar").parse().unwrap(),
        );
        let result = extract_session(&headers).unwrap();
        assert_eq!(result, id);
    }

    #[test]
    fn extract_session_no_cookie_header() {
        let headers = HeaderMap::new();
        assert!(extract_session(&headers).is_err());
    }

    #[test]
    fn extract_session_no_whoop_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "other=value".parse().unwrap());
        assert!(extract_session(&headers).is_err());
    }

    #[test]
    fn extract_session_invalid_uuid() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "whoop_session=not-a-uuid".parse().unwrap());
        assert!(extract_session(&headers).is_err());
    }
}
