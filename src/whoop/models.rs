use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

impl TokenResponse {
    pub fn into_token_pair(self) -> TokenPair {
        TokenPair {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at: Utc::now() + chrono::Duration::seconds(self.expires_in),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PaginationParams {
    pub start: Option<String>,
    pub end: Option<String>,
    pub limit: Option<u32>,
    #[serde(rename = "nextToken")]
    pub next_token: Option<String>,
}

impl PaginationParams {
    pub fn to_query_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = Vec::new();
        if let Some(ref start) = self.start {
            pairs.push(("start", start.clone()));
        }
        if let Some(ref end) = self.end {
            pairs.push(("end", end.clone()));
        }
        if let Some(limit) = self.limit {
            pairs.push(("limit", limit.to_string()));
        }
        if let Some(ref next_token) = self.next_token {
            pairs.push(("nextToken", next_token.clone()));
        }
        pairs
    }
}
