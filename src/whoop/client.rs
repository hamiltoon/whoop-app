use axum::http::StatusCode;
use serde_json::Value;

use crate::state::{AppState, SessionId};
use crate::whoop::auth;
use crate::whoop::models::PaginationParams;

const BASE_URL: &str = "https://api.prod.whoop.com/developer";

pub struct WhoopClient<'a> {
    state: &'a AppState,
    session_id: SessionId,
}

pub enum ApiResult {
    Success(Value),
    Error { status: StatusCode, body: Value },
}

impl<'a> WhoopClient<'a> {
    pub fn new(state: &'a AppState, session_id: SessionId) -> Self {
        Self { state, session_id }
    }

    pub async fn get(&self, path: &str, params: Option<&PaginationParams>) -> Result<ApiResult, AppError> {
        let result = self.do_get(path, params).await?;

        // If 401, try refreshing the token and retrying
        if let ApiResult::Error { ref status, .. } = result {
            if *status == StatusCode::UNAUTHORIZED {
                if self.try_refresh().await.is_ok() {
                    return self.do_get(path, params).await;
                } else {
                    return Err(AppError::TokenRefreshFailed);
                }
            }
        }

        Ok(result)
    }

    async fn do_get(&self, path: &str, params: Option<&PaginationParams>) -> Result<ApiResult, AppError> {
        let access_token = {
            let tokens = self.state.tokens.read().await;
            let pair = tokens.get(&self.session_id).ok_or(AppError::Unauthorized)?;
            pair.access_token.clone()
        };

        let url = format!("{BASE_URL}{path}");
        let mut req = self.state.http_client.get(&url)
            .bearer_auth(&access_token);

        if let Some(p) = params {
            let pairs = p.to_query_pairs();
            if !pairs.is_empty() {
                req = req.query(&pairs);
            }
        }

        let resp = req.send().await.map_err(|e| AppError::InternalError(e.to_string()))?;
        let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body: Value = resp.json().await.unwrap_or_else(|_| serde_json::json!({"error": "Failed to parse Whoop response"}));

        if status.is_success() {
            Ok(ApiResult::Success(body))
        } else {
            Ok(ApiResult::Error { status, body })
        }
    }

    async fn try_refresh(&self) -> Result<(), ()> {
        let refresh = {
            let tokens = self.state.tokens.read().await;
            let pair = tokens.get(&self.session_id).ok_or(())?;
            pair.refresh_token.clone()
        };

        let new_pair = auth::refresh_token(
            &self.state.http_client,
            &self.state.config,
            &refresh,
        )
        .await
        .map_err(|_| ())?;

        let mut tokens = self.state.tokens.write().await;
        tokens.insert(self.session_id, new_pair);
        Ok(())
    }
}

// Error type
#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    WhoopApiError { status: StatusCode, body: Value },
    TokenRefreshFailed,
    InternalError(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized: no valid session".to_string()),
            AppError::WhoopApiError { status, body } => {
                return (status, axum::Json(body)).into_response();
            }
            AppError::TokenRefreshFailed => (StatusCode::UNAUTHORIZED, "Token refresh failed, please re-authenticate".to_string()),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, axum::Json(serde_json::json!({"error": message}))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn unauthorized_error_returns_401() {
        let resp = AppError::Unauthorized.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn token_refresh_failed_returns_401() {
        let resp = AppError::TokenRefreshFailed.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn internal_error_returns_500() {
        let resp = AppError::InternalError("boom".to_string()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn whoop_api_error_returns_upstream_status() {
        let resp = AppError::WhoopApiError {
            status: StatusCode::FORBIDDEN,
            body: serde_json::json!({"msg": "forbidden"}),
        }
        .into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn app_error_debug_format() {
        let err = AppError::Unauthorized;
        assert!(format!("{:?}", err).contains("Unauthorized"));
    }
}
