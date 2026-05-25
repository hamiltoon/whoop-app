use crate::config::Config;
use crate::whoop::models::{TokenPair, TokenResponse};

const AUTH_URL: &str = "https://api.prod.whoop.com/oauth/oauth2/auth";
const TOKEN_URL: &str = "https://api.prod.whoop.com/oauth/oauth2/token";

const SCOPES: &str = "read:recovery read:cycles read:workout read:sleep read:profile read:body_measurement";

pub fn build_auth_url(config: &Config) -> String {
    let state = uuid::Uuid::new_v4().to_string();
    format!(
        "{AUTH_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={state}",
        config.client_id,
        urlencoding(&config.redirect_uri),
        urlencoding(SCOPES),
    )
}

pub async fn exchange_code(
    client: &reqwest::Client,
    config: &Config,
    code: &str,
) -> Result<TokenPair, String> {
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", config.redirect_uri.as_str()),
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {body}"));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))?;

    Ok(token_resp.into_token_pair())
}

pub async fn refresh_token(
    client: &reqwest::Client,
    config: &Config,
    refresh: &str,
) -> Result<TokenPair, String> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh),
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err("Token refresh failed".to_string());
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {e}"))?;

    Ok(token_resp.into_token_pair())
}

fn urlencoding(s: &str) -> String {
    s.replace(':', "%3A")
        .replace('/', "%2F")
        .replace(' ', "%20")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            client_id: "my_client_id".to_string(),
            client_secret: "my_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            port: 3000,
        }
    }

    #[test]
    fn auth_url_contains_client_id() {
        let url = build_auth_url(&test_config());
        assert!(url.contains("client_id=my_client_id"));
    }

    #[test]
    fn auth_url_contains_encoded_redirect_uri() {
        let url = build_auth_url(&test_config());
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fcallback"));
    }

    #[test]
    fn auth_url_contains_response_type_code() {
        let url = build_auth_url(&test_config());
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn auth_url_contains_encoded_scopes() {
        let url = build_auth_url(&test_config());
        assert!(url.contains("scope=read%3Arecovery%20read%3Acycles%20read%3Aworkout%20read%3Asleep%20read%3Aprofile%20read%3Abody_measurement"));
    }

    #[test]
    fn auth_url_starts_with_correct_base() {
        let url = build_auth_url(&test_config());
        assert!(url.starts_with(AUTH_URL));
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        assert_eq!(urlencoding("http://example.com/path"), "http%3A%2F%2Fexample.com%2Fpath");
        assert_eq!(urlencoding("a b c"), "a%20b%20c");
        assert_eq!(urlencoding("plain"), "plain");
    }
}
