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
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

impl TokenResponse {
    pub fn into_token_pair(self) -> TokenPair {
        TokenPair {
            access_token: self.access_token,
            refresh_token: self.refresh_token.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_response_deserialization() {
        let json = r#"{"access_token":"abc","refresh_token":"def","expires_in":3600}"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "abc");
        assert_eq!(resp.refresh_token.as_deref(), Some("def"));
        assert_eq!(resp.expires_in, 3600);
    }

    #[test]
    fn token_response_into_token_pair() {
        let resp = TokenResponse {
            access_token: "acc".to_string(),
            refresh_token: Some("ref".to_string()),
            expires_in: 7200,
            token_type: None,
            scope: None,
        };
        let pair = resp.into_token_pair();
        assert_eq!(pair.access_token, "acc");
        assert_eq!(pair.refresh_token, "ref");
        // expires_at should be roughly 7200 seconds from now
        let diff = pair.expires_at - Utc::now();
        assert!(diff.num_seconds() > 7190 && diff.num_seconds() <= 7200);
    }

    #[test]
    fn token_response_missing_field_fails() {
        let json = r#"{"access_token":"abc","refresh_token":"def"}"#;
        assert!(serde_json::from_str::<TokenResponse>(json).is_err());
    }

    #[test]
    fn token_pair_roundtrip_serde() {
        let pair = TokenPair {
            access_token: "a".to_string(),
            refresh_token: "r".to_string(),
            expires_at: Utc::now(),
        };
        let json = serde_json::to_string(&pair).unwrap();
        let back: TokenPair = serde_json::from_str(&json).unwrap();
        assert_eq!(back.access_token, "a");
        assert_eq!(back.refresh_token, "r");
    }

    #[test]
    fn pagination_params_empty() {
        let p = PaginationParams::default();
        assert!(p.to_query_pairs().is_empty());
    }

    #[test]
    fn pagination_params_all_fields() {
        let p = PaginationParams {
            start: Some("2024-01-01".to_string()),
            end: Some("2024-12-31".to_string()),
            limit: Some(25),
            next_token: Some("tok123".to_string()),
        };
        let pairs = p.to_query_pairs();
        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0], ("start", "2024-01-01".to_string()));
        assert_eq!(pairs[1], ("end", "2024-12-31".to_string()));
        assert_eq!(pairs[2], ("limit", "25".to_string()));
        assert_eq!(pairs[3], ("nextToken", "tok123".to_string()));
    }

    #[test]
    fn pagination_params_partial() {
        let p = PaginationParams {
            start: None,
            end: None,
            limit: Some(10),
            next_token: None,
        };
        let pairs = p.to_query_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("limit", "10".to_string()));
    }

    #[test]
    fn pagination_params_deserialize_with_next_token() {
        let json = r#"{"start":"2024-01-01","nextToken":"abc"}"#;
        let p: PaginationParams = serde_json::from_str(json).unwrap();
        assert_eq!(p.start.as_deref(), Some("2024-01-01"));
        assert_eq!(p.next_token.as_deref(), Some("abc"));
        assert!(p.end.is_none());
        assert!(p.limit.is_none());
    }
}
