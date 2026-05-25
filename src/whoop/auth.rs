use crate::config::Config;
use crate::whoop::models::{TokenPair, TokenResponse};

const AUTH_URL: &str = "https://api.prod.whoop.com/oauth/oauth2/auth";
const TOKEN_URL: &str = "https://api.prod.whoop.com/oauth/oauth2/token";

const SCOPES: &str = "read:recovery read:cycles read:workout read:sleep read:profile read:body_measurement";

pub fn build_auth_url(config: &Config) -> String {
    format!(
        "{AUTH_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&state=whoop",
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
